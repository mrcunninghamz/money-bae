/*
 * What's left here is purely decorative placeholder content for the home
 * page (next-up bills, the net-by-cycle spark chart, and the current-cycle
 * label strings) — Bill/Income/Ledger/PTO/Holiday all moved to real data via
 * servers/api (see data/api.ts and data/store.tsx). This file used to hold
 * mock arrays for all of those; they were removed once nothing referenced
 * them anymore.
 */

export interface NextUpEntry {
  due: string
  name: string
  amount: number
}

export const nextUp: NextUpEntry[] = [
  { due: '28/12', name: 'Amex', amount: 684.2 },
  { due: '01/01', name: 'Mortgage', amount: 1596.27 },
  { due: '01/01', name: 'GX460', amount: 948.41 },
  { due: '05/01', name: 'Centerpoint', amount: 22.14 },
]

// SparkPoint is the shared shape for SparkBars — home page cycles come from
// GET /ledgers/history now, but the type stays here alongside the rest of
// this file's decorative home-page content.
export interface SparkPoint {
  label: string
  value: number
}
