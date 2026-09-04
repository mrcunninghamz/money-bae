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

export interface SparkPoint {
  label: string
  value: number
}

export const sparkSeries: SparkPoint[] = [
  { label: 'Jul1', value: 3492 },
  { label: 'Jul2', value: 2231 },
  { label: 'Aug1', value: 2662 },
  { label: 'Aug2', value: 892 },
  { label: 'Sep1', value: 2583 },
  { label: 'Sep2', value: 2856 },
  { label: 'Oct1', value: 3550 },
  { label: 'Nov2', value: 3109 },
]

export const CURRENT_CYCLE_LABEL = 'December P1'
export const CURRENT_CYCLE_DATE = '12/12/2025'
export const DAYS_TO_PAYDAY = 7
