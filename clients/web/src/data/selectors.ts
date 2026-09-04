import type { SparkPoint } from '#/data/mockData'

// point.value is a net percentage (0-100, roughly) from GET /ledgers/history
// — clamp before scaling so an over-100% or negative cycle doesn't blow out
// the bar height.
export function sparkBarHeights(
  series: SparkPoint[],
): Array<{ label: string; height: string }> {
  return series.map((point) => ({
    label: point.label,
    height: `${Math.round((Math.min(100, Math.max(0, point.value)) / 100) * 68)}px`,
  }))
}
