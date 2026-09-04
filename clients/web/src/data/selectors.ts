import type { SparkPoint } from '#/data/mockData'

// point.value is a cycle's raw net dollar amount from GET /ledgers/history.
// Scale positive and negative bars independently against the series' own
// ceiling (highest net) and floor (lowest net) — the tallest positive bar
// always reaches 100% of the upper half, the deepest negative bar always
// reaches 100% of the lower half — rather than a fixed percentage scale
// that breaks down whenever a cycle has zero available funds.
export function sparkBarHeights(series: SparkPoint[]): Array<{
  label: string
  positiveHeightPct: number
  negativeHeightPct: number
}> {
  const values = series.map((point) => point.value)
  const ceiling = Math.max(0, ...values)
  const floor = Math.min(0, ...values)
  return series.map((point) => ({
    label: point.label,
    positiveHeightPct:
      point.value > 0 && ceiling > 0 ? (point.value / ceiling) * 100 : 0,
    negativeHeightPct:
      point.value < 0 && floor < 0 ? (point.value / floor) * 100 : 0,
  }))
}
