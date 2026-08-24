SOURCES=$(shell python3 scripts/read-config.py --sources )
FAMILY=$(shell python3 scripts/read-config.py --family )

help:
	@echo "###"
	@echo "# Build targets for $(FAMILY)"
	@echo "###"
	@echo
	@echo "  make build:  Builds the fonts and places them in the fonts/ directory"
	@echo "  make test:   Tests the fonts with fontspector"
	@echo "  make proof:  Creates HTML proof documents in the proof/ directory"
	@echo "  make images: Creates PNG specimen images in the documentation/ directory"
	@echo

build: build.stamp

venv: venv/touchfile

venv-pixel: venv-pixel/touchfile

customize: venv
	. venv/bin/activate; python3 scripts/customize.py

build.stamp: venv venv-pixel sources/config-Zen.yaml $(SOURCES)
	rm -rf fonts zen-font zen-font.zip
	# Zen Pixel uses a virtual master, which the released gftools in
	# requirements.txt can't build (it fails with "No final targets"). Build it
	# with the dev gftools in venv-pixel that has the virtual-master fix;
	# everything else uses venv.
	@for config in sources/config*.yaml; do \
		if [ "$$config" = "sources/config-ZenPixel.yaml" ]; then \
			( . venv-pixel/bin/activate && gftools builder "$$config" ); \
		else \
			( . venv/bin/activate && gftools builder "$$config" ); \
		fi; \
	done
	# Zen Pixel's static instances and webfonts are hand-exported from Glyphs
	# and committed (buildStatic:false means gftools only builds the variable, so
	# the clean `rm -rf fonts` above drops them). Restore the committed assets the
	# release zip and npm package consume.
	git checkout -- fonts/ZenPixel/otf fonts/ZenPixel/ttf fonts/ZenPixel/webfonts
	. venv/bin/activate; python3 packages/zen/scripts/name.py
	$(MAKE) copy-npm-fonts
	$(MAKE) create-release-zip
	touch build.stamp

copy-npm-fonts:
	# Clear any pre-existing build artifacts
	rm -rf packages/zen/dist/fonts
	# Copy over the relevant font files
	mkdir -p packages/zen/dist/fonts/zen-sans packages/zen/dist/fonts/zen-mono packages/zen/dist/fonts/zen-pixel
	cp fonts/Zen/ttf/*.ttf packages/zen/dist/fonts/zen-sans/
	cp fonts/Zen/webfonts/*.woff2 packages/zen/dist/fonts/zen-sans/
	cp fonts/Zen/variable/*.ttf packages/zen/dist/fonts/zen-sans/
	cp fonts/ZenMono/ttf/*.ttf packages/zen/dist/fonts/zen-mono/
	cp fonts/ZenMono/webfonts/*.woff2 packages/zen/dist/fonts/zen-mono/
	cp fonts/ZenMono/variable/*.ttf packages/zen/dist/fonts/zen-mono/
	cp fonts/ZenPixel/webfonts/*.woff2 packages/zen/dist/fonts/zen-pixel/
	# Zen Pixel's five cuts are one axis, so the variable carries all five and
	# morphs between them. gftools writes it as ttf only; the web wants woff2.
	. venv/bin/activate; python3 -c "from fontTools.ttLib import TTFont; f=TTFont('fonts/ZenPixel/variable/ZenPixel[ELSH].ttf'); f.flavor='woff2'; f.save('packages/zen/dist/fonts/zen-pixel/ZenPixel-Variable.woff2')"
	cp 'fonts/ZenPixel/variable/ZenPixel[ELSH].ttf' packages/zen/dist/fonts/zen-pixel/ZenPixel-Variable.ttf
	# Apparently there is a naming mismatch between the font files for npm distribution and the actual font files,
	# so we need to rename them to the correct names.
	cd packages/zen/dist/fonts/zen-sans && \
		mv Zen-ExtraLight.ttf Zen-UltraLight.ttf && \
		mv Zen-ExtraLight.woff2 Zen-UltraLight.woff2 && \
		mv Zen-ExtraBold.ttf Zen-UltraBlack.ttf && \
		mv Zen-ExtraBold.woff2 Zen-UltraBlack.woff2 && \
		mv 'Zen[wght].ttf' Zen-Variable.ttf && \
		mv 'Zen[wght].woff2' Zen-Variable.woff2
	cd packages/zen/dist/fonts/zen-mono && \
		mv ZenMono-ExtraLight.ttf ZenMono-UltraLight.ttf && \
		mv ZenMono-ExtraLight.woff2 ZenMono-UltraLight.woff2 && \
		mv ZenMono-ExtraBold.ttf ZenMono-UltraBlack.ttf && \
		mv ZenMono-ExtraBold.woff2 ZenMono-UltraBlack.woff2 && \
		mv 'ZenMono[wght].ttf' ZenMono-Variable.ttf && \
		mv 'ZenMono[wght].woff2' ZenMono-Variable.woff2

create-release-zip:
	mkdir -p zen-font
	cp -r fonts/* zen-font/
	cp documentation/DESCRIPTION.en_us.html zen-font/ || true
	cp documentation/article/ARTICLE.en_us.html zen-font/ || true
	cp LICENSE.txt zen-font/
	zip -r zen-font.zip zen-font
	rm -rf zen-font

venv/touchfile: requirements.txt
	test -d venv || python3 -m venv venv
	. venv/bin/activate; pip install -Ur requirements.txt
	touch venv/touchfile

# Zen Pixel's virtual-master support only exists in an unreleased gftools dev
# build (Simon Cozens' fix). Pin the exact commit for reproducibility; revisit
# once it ships in an official gftools release and we can fold it into venv.
GFTOOLS_PIXEL_REF = 47ec3706b

venv-pixel/touchfile: Makefile
	test -d venv-pixel || python3 -m venv venv-pixel
	. venv-pixel/bin/activate; pip install "gftools @ git+https://github.com/googlefonts/gftools@$(GFTOOLS_PIXEL_REF)"
	touch venv-pixel/touchfile

test: build.stamp
	which fontspector || (echo "fontspector not found. Please install it with 'cargo install fontspector'." && exit 1)
	TOCHECK=$$(find fonts/Zen/variable -type f 2>/dev/null); mkdir -p out/ out/fontspector; fontspector --profile googlefonts -l warn --full-lists --succinct --html out/fontspector/ZenVF-fontspector-report.html --ghmarkdown out/fontspector/ZenVF-fontspector-report.md --badges out/badges $$TOCHECK  || echo '::warning file=sources/config-Zen.yaml,title=fontspector failures::The fontspector QA check reported errors in your font. Please check the generated report.'
	TOCHECK=$$(find fonts/Zen/ttf -type f 2>/dev/null); mkdir -p out/ out/fontspector; fontspector --profile googlefonts -l warn --full-lists --succinct --html out/fontspector/Zen-fontspector-report.html --ghmarkdown out/fontspector/Zen-fontspector-report.md --badges out/badges $$TOCHECK  || echo '::warning file=sources/config-Zen.yaml,title=fontspector failures::The fontspector QA check reported errors in your font. Please check the generated report.'
	TOCHECK=$$(find fonts/ZenMono/variable -type f 2>/dev/null); mkdir -p out/ out/fontspector; fontspector --profile googlefonts -l warn --full-lists --succinct --html out/fontspector/ZenMonoVF-fontspector-report.html --ghmarkdown out/fontspector/ZenMonoVF-fontspector-report.md --badges out/badges $$TOCHECK  || echo '::warning file=sources/config-ZenMono.yaml,title=fontspector failures::The fontspector QA check reported errors in your font. Please check the generated report.'
	TOCHECK=$$(find fonts/ZenMono/ttf -type f 2>/dev/null); mkdir -p out/ out/fontspector; fontspector --profile googlefonts -l warn --full-lists --succinct --html out/fontspector/ZenMono-fontspector-report.html --ghmarkdown out/fontspector/ZenMono-fontspector-report.md --badges out/badges $$TOCHECK  || echo '::warning file=sources/config-ZenMono.yaml,title=fontspector failures::The fontspector QA check reported errors in your font. Please check the generated report.'
	TOCHECK=$$(find fonts/ZenPixel/ttf -type f 2>/dev/null); mkdir -p out/ out/fontspector; fontspector --profile googlefonts -l warn --full-lists --succinct --html out/fontspector/ZenPixel-fontspector-report.html --ghmarkdown out/fontspector/ZenPixel-fontspector-report.md --badges out/badges $$TOCHECK  || echo '::warning file=sources/config-ZenPixel.yaml,title=fontspector failures::The fontspector QA check reported errors in your font. Please check the generated report.'

proof: venv build.stamp
	TOCHECK=$$(find fonts/Zen/variable -type f 2>/dev/null); if [ -z "$$TOCHECK" ]; then TOCHECK=$$(find fonts/Zen/ttf -type f 2>/dev/null); fi ; . venv/bin/activate; mkdir -p out/ out/proof; diffenator2 proof $$TOCHECK -o out/proof

images: venv $(DRAWBOT_OUTPUT)

%.png: %.py build.stamp
	. venv/bin/activate; python3 $< --output $@

clean:
	rm -rf venv venv-pixel
	find . -name "*.pyc" -delete

update-project-template:
	npx update-template https://github.com/googlefonts/googlefonts-project-template/

update: venv
	venv/bin/pip install --upgrade pip-tools
	# See https://pip-tools.readthedocs.io/en/latest/#a-note-on-resolvers for
	# the `--resolver` flag below.
	venv/bin/pip-compile --upgrade --verbose --resolver=backtracking requirements.in
	venv/bin/pip-sync requirements.txt

	git commit -m "Update requirements" requirements.txt
	git push
