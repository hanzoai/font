export interface ZenPreset {
  wght: number
  scaleX: number
  track: number
  note: string
  fits?: string
  /** Coverage: whole-mark pixel difference as a fraction of target ink. */
  residual?: number
  /** The worst single feature — stem, bar — as a ratio to the target's. */
  within?: number
}
export declare const PRESETS: Record<'air' | 'book' | 'medium' | 'wide' | 'round', ZenPreset>
export declare const BASEL_XHEIGHT_FACTOR: number
export declare function preset(name: keyof typeof PRESETS): Record<string, string | number>
