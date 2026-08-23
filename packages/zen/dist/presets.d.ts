export interface ZenPreset {
  wght: number
  scaleX: number
  track: number
  note: string
  fits?: string
  residual?: number
}
export declare const PRESETS: Record<'air' | 'book' | 'medium' | 'wide' | 'round', ZenPreset>
export declare const XHEIGHT_FACTOR: number
export declare function preset(name: keyof typeof PRESETS): Record<string, string | number>
