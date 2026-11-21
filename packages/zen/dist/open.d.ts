export interface Open {
  family: string
  /** The file name under open/: <slug>.css names this family's faces, beside its woff2. */
  slug: string
  role: 'sans' | 'serif' | 'display' | 'mono'
  /** Its OFL.txt. */
  licence: string
}
export declare const OPEN: readonly Open[]
