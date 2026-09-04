import type { SparkPoint } from '#/data/mockData'

export function sparkBarHeights(
  series: SparkPoint[],
): Array<{ label: string; height: string }> {
  return series.map((point) => ({
    label: point.label,
    height: `${Math.round((point.value / 3600) * 68)}px`,
  }))
}
