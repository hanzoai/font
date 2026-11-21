export interface Open {
  family: string
  /** The file name under open/: <slug>.css names this family's faces, beside its woff2. */
  slug: string
  role: 'sans' | 'serif' | 'display' | 'mono'
  /** Every weight a face of it sets, in steps of 100. */
  weights: readonly number[]
  italic: boolean
  /** Its OFL.txt. */
  licence: string
}
export declare const OPEN: readonly Open[]
