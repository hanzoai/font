//! Identity — who a built face says it is.
//!
//! Zen's outlines are a derivative of Geist, and the built faces carried Geist's
//! `name` table long after the family was renamed. Nothing visual can see that:
//! the filenames read `Zen-Regular.ttf`, the glyphs draw correctly, and only a
//! font picker, a `local()` match or a licence audit reads the table that says
//! otherwise. 390 of 521 built files in this estate were in that state.
//!
//! This RELABELS rather than rebuilds. A table-by-table diff of the corrected
//! build against the stale one shows only `name` and `head` differ — every
//! outline, metric and layout table is byte-identical — so rebuilding from the
//! Glyphs sources to fix a label would risk moving the drawing to fix the
//! caption.
//!
//! WHAT IS KEPT, DELIBERATELY. nameID 0 still carries
//! "Copyright 2024 The Geist Project Authors" beside Hanzo's own line. SIL OFL
//! 1.1 §1 requires copyright notices to be retained in ALL copies, so stripping
//! it would turn every one of these files into an unlicensed derivative. Zen is
//! a derivative, and the compliant way to ship one is Hanzo as designer and
//! manufacturer OVER a retained upstream copyright — which is what the published
//! build already does. Everything that says who MADE it is rewritten; the one
//! line that says where it CAME FROM stays.
//!
//! WHY THERE IS NO SERIALISER DEPENDENCY. `write-fonts` is the obvious answer and
//! does not compile here: its `pens` module is unconditional and calls
//! `Rect::union_pt((x, y).into())`, which newer kurbo made ambiguous by adding
//! more `From<(f64, f64)>` impls. Replacing ONE table in an sfnt is a directory
//! rebuild and two checksums — the same trade this crate already took for the
//! rasteriser, and it keeps the wasm build free of a serialiser it never calls.

/// Upstream family → ours. Longest first, so "Geist Mono" is matched before the
/// "Geist" inside it; shortest-first renames it to "Zen Mono" only by luck of
/// iteration order and to "Zen" the rest of the time.
const FAMILIES: [(&str, &str); 7] = [
    ("Geist Pixel Circle", "Zen Pixel Circle"),
    ("Geist Pixel Triangle", "Zen Pixel Triangle"),
    ("Geist Pixel Square", "Zen Pixel Square"),
    ("Geist Pixel Grid", "Zen Pixel Grid"),
    ("Geist Pixel Line", "Zen Pixel Line"),
    ("Geist Mono", "Zen Mono"),
    ("Geist", "Zen"),
];

const MAKER: &str = "Hanzo AI, Inc.";
const HOME: &str = "https://hanzo.ai";
/// The four-byte OS/2 vendor tag. Vercel's is `VRCL`.
const VENDOR: &[u8; 4] = b"HNZO";

/// Records carrying the family: family (1), unique id (3), full (4), PostScript
/// (6), typographic family (16), and the variable-instance names (18, 21, 25).
const FAMILY_IDS: [u16; 8] = [1, 3, 4, 6, 16, 18, 21, 25];

/// Records naming the maker rather than the family. nameID 0 is absent on
/// purpose — see the module note.
fn maker(id: u16) -> Option<&'static str> {
    match id {
        8 | 9 => Some(MAKER),   // manufacturer, designer
        11 | 12 => Some(HOME),  // vendor URL, designer URL
        _ => None,
    }
}

fn renamed(text: &str) -> String {
    let mut s = text.to_string();
    for (from, to) in FAMILIES {
        if s.contains(from) {
            s = s.replace(from, to);
        }
    }
    s
}

/// What one call changed, so a caller reports rather than asserts.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changed {
    pub records: usize,
    pub vendor: bool,
}

impl Changed {
    pub fn any(&self) -> bool {
        self.records > 0 || self.vendor
    }
}

fn be16(b: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([b[at], b[at + 1]])
}
fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// One decoded name record: the four selector fields, and the string as UTF-8.
struct Record {
    platform: u16,
    encoding: u16,
    language: u16,
    name: u16,
    text: String,
}

/// A name record's bytes are UTF-16BE on platform 3 (Windows) and Mac Roman —
/// close enough to Latin-1 for the ASCII these carry — on platform 1.
fn decode(platform: u16, raw: &[u8]) -> Option<String> {
    if platform == 3 {
        if raw.len() % 2 != 0 {
            return None;
        }
        let units: Vec<u16> = raw.chunks_exact(2).map(|c| be16(c, 0)).collect();
        String::from_utf16(&units).ok()
    } else {
        Some(raw.iter().map(|&b| b as char).collect())
    }
}

fn encode(platform: u16, text: &str) -> Vec<u8> {
    if platform == 3 {
        text.encode_utf16().flat_map(u16::to_be_bytes).collect()
    } else {
        text.chars().map(|c| c as u8).collect()
    }
}

/// Build a format-0 `name` table from records, pooling identical strings.
fn build_name(records: &[Record]) -> Vec<u8> {
    let count = records.len();
    let store_at = 6 + count * 12;
    let mut store: Vec<u8> = Vec::new();
    let mut out = Vec::with_capacity(store_at + 64);
    out.extend_from_slice(&0u16.to_be_bytes()); // format 0
    out.extend_from_slice(&(count as u16).to_be_bytes());
    out.extend_from_slice(&(store_at as u16).to_be_bytes());

    for r in records {
        let bytes = encode(r.platform, &r.text);
        // Pool: an identical string is stored once. Fonts repeat the family name
        // across a dozen records, so this is most of the table.
        let offset = match store
            .windows(bytes.len())
            .position(|w| w == bytes.as_slice())
        {
            Some(at) => at,
            None => {
                let at = store.len();
                store.extend_from_slice(&bytes);
                at
            }
        };
        out.extend_from_slice(&r.platform.to_be_bytes());
        out.extend_from_slice(&r.encoding.to_be_bytes());
        out.extend_from_slice(&r.language.to_be_bytes());
        out.extend_from_slice(&r.name.to_be_bytes());
        out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        out.extend_from_slice(&(offset as u16).to_be_bytes());
    }
    out.extend_from_slice(&store);
    out
}

/// The sfnt table checksum: the big-endian u32 sum of the table, zero-padded to
/// a four-byte boundary, wrapping.
fn checksum(table: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut i = 0;
    while i < table.len() {
        let mut word = [0u8; 4];
        for (k, slot) in word.iter_mut().enumerate() {
            if i + k < table.len() {
                *slot = table[i + k];
            }
        }
        sum = sum.wrapping_add(u32::from_be_bytes(word));
        i += 4;
    }
    sum
}

/// Rewrite `font`'s identity, answering the new bytes and what moved.
///
/// Returns the input UNCHANGED when there is nothing to do, so a run over an
/// already-correct tree is a no-op rather than a re-serialisation — which
/// matters, because re-serialising rewrites `head.checkSumAdjustment` and would
/// make every file look modified.
pub fn rewrite(font: &[u8]) -> Result<(Vec<u8>, Changed), String> {
    if font.len() < 12 {
        return Err("not an sfnt".into());
    }
    let tag = be32(font, 0);
    if tag != 0x0001_0000 && tag != 0x4F54_544F && tag != 0x7472_7565 {
        return Err(format!("not a ttf or otf (sfnt tag {tag:#x})"));
    }
    let num = be16(font, 4) as usize;

    // The directory, as (tag, offset, length), in the order the file states.
    let mut dir: Vec<([u8; 4], usize, usize)> = Vec::with_capacity(num);
    for i in 0..num {
        let at = 12 + i * 16;
        if at + 16 > font.len() {
            return Err("truncated table directory".into());
        }
        let mut t = [0u8; 4];
        t.copy_from_slice(&font[at..at + 4]);
        dir.push((t, be32(font, at + 8) as usize, be32(font, at + 12) as usize));
    }

    let find = |want: &[u8; 4]| dir.iter().find(|(t, _, _)| t == want).copied();
    let (_, name_at, name_len) = find(b"name").ok_or("no name table")?;
    if name_at + name_len > font.len() {
        return Err("name table runs past the end".into());
    }
    let name = &font[name_at..name_at + name_len];

    // Decode every record.
    let count = be16(name, 2) as usize;
    let store = be16(name, 4) as usize;
    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let at = 6 + i * 12;
        let platform = be16(name, at);
        let len = be16(name, at + 8) as usize;
        let off = be16(name, at + 10) as usize;
        let from = store + off;
        if from + len > name.len() {
            return Err("name record runs past the table".into());
        }
        let Some(text) = decode(platform, &name[from..from + len]) else {
            continue;
        };
        records.push(Record {
            platform,
            encoding: be16(name, at + 2),
            language: be16(name, at + 4),
            name: be16(name, at + 6),
            text,
        });
    }

    let mut what = Changed::default();
    for r in records.iter_mut() {
        let next = match maker(r.name) {
            Some(v) => v.to_string(),
            None if FAMILY_IDS.contains(&r.name) => renamed(&r.text),
            None => continue,
        };
        if next != r.text {
            r.text = next;
            what.records += 1;
        }
    }

    let vendor_at = find(b"OS/2").map(|(_, at, _)| at + 58);
    if let Some(at) = vendor_at {
        if at + 4 <= font.len() && &font[at..at + 4] != VENDOR {
            what.vendor = true;
        }
    }

    if !what.any() {
        return Ok((font.to_vec(), what));
    }

    let new_name = build_name(&records);

    // Reassemble. Tables keep their file order; only `name` changes size, and the
    // OS/2 vendor tag is a fixed-width overwrite.
    let mut out = vec![0u8; 12 + num * 16];
    out[..12].copy_from_slice(&font[..12]);
    let mut head_at_out = None;

    for (i, (t, at, len)) in dir.iter().enumerate() {
        let body: Vec<u8> = if t == b"name" {
            new_name.clone()
        } else {
            let mut b = font[*at..*at + *len].to_vec();
            if t == b"OS/2" && b.len() >= 62 {
                b[58..62].copy_from_slice(VENDOR);
            }
            b
        };
        while out.len() % 4 != 0 {
            out.push(0);
        }
        let offset = out.len();
        if t == b"head" {
            head_at_out = Some(offset);
        }
        let sum = checksum(&body);
        out.extend_from_slice(&body);

        let rec = 12 + i * 16;
        out[rec..rec + 4].copy_from_slice(t);
        out[rec + 4..rec + 8].copy_from_slice(&sum.to_be_bytes());
        out[rec + 8..rec + 12].copy_from_slice(&(offset as u32).to_be_bytes());
        out[rec + 12..rec + 16].copy_from_slice(&(*len as u32).to_be_bytes());
        if t == b"name" {
            out[rec + 12..rec + 16].copy_from_slice(&(new_name.len() as u32).to_be_bytes());
        }
    }

    // head.checkSumAdjustment is 0xB1B0AFBA minus the checksum of the WHOLE file
    // computed with that field zeroed, so it is written last and in two passes.
    if let Some(head) = head_at_out {
        out[head + 8..head + 12].copy_from_slice(&0u32.to_be_bytes());
        let whole = checksum(&out);
        let adj = 0xB1B0_AFBAu32.wrapping_sub(whole);
        out[head + 8..head + 12].copy_from_slice(&adj.to_be_bytes());
    }

    Ok((out, what))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The renaming is longest-first, so the family inside another family is not
    /// eaten. This is the whole reason FAMILIES is ordered rather than a map.
    #[test]
    fn mono_is_not_renamed_as_sans() {
        assert_eq!(renamed("Geist Mono"), "Zen Mono");
        assert_eq!(renamed("Geist Mono Black"), "Zen Mono Black");
        // A PostScript name carries no space, so the bare "Geist" arm is what
        // renames it — and that is correct rather than incidental: the published
        // face is ZenMono-Bold. Asserting it stayed put was this test's own error.
        assert_eq!(renamed("GeistMono-Bold"), "ZenMono-Bold");
        assert_eq!(renamed("Geist"), "Zen");
        assert_eq!(renamed("Geist Pixel Circle"), "Zen Pixel Circle");
    }

    /// nameID 0 is not in the maker set, and that is a licence requirement rather
    /// than an oversight: OFL 1.1 §1 keeps copyright notices in every copy.
    #[test]
    fn the_copyright_is_never_a_maker_field() {
        assert!(maker(0).is_none());
        assert_eq!(maker(9), Some(MAKER));
        assert_eq!(maker(11), Some(HOME));
    }

    /// A real face, round-tripped: the identity moves and the outlines do not.
    #[test]
    fn a_real_face_keeps_its_glyphs() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fonts/Zen/ttf/Zen-Regular.ttf");
        let Ok(bytes) = std::fs::read(path) else {
            return; // the built tree is not always present
        };
        let (out, what) = rewrite(&bytes).expect("rewrite");
        assert!(what.records > 0, "a stale face has records to move");

        // glyf is the drawing. It must survive byte for byte.
        let glyf = |f: &[u8]| -> Vec<u8> {
            let num = u16::from_be_bytes([f[4], f[5]]) as usize;
            for i in 0..num {
                let at = 12 + i * 16;
                if &f[at..at + 4] == b"glyf" {
                    let off = be32(f, at + 8) as usize;
                    let len = be32(f, at + 12) as usize;
                    return f[off..off + len].to_vec();
                }
            }
            Vec::new()
        };
        assert_eq!(glyf(&bytes), glyf(&out), "the outlines moved");

        // And running it again is a no-op, so a sweep is idempotent.
        let (again, twice) = rewrite(&out).expect("second");
        assert!(!twice.any(), "already-correct input must not be rewritten");
        assert_eq!(again, out);
    }
}
