/*
 * What's left here is purely decorative placeholder content for the home
 * page (the net-by-cycle spark chart) — Bill/Income/Ledger/PTO/Holiday all
 * moved to real data via servers/api (see data/api.ts and data/store.tsx).
 * This file used to hold mock arrays for all of those; they were removed
 * once nothing referenced them anymore.
 */

// SparkPoint is the shared shape for SparkBars — home page cycles come from
// GET /ledgers/history now, but the type stays here alongside the rest of
// this file's decorative home-page content.
export interface SparkPoint {
  label: string
  value: number
}
