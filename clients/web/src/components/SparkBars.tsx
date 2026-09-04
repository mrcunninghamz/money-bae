import { sparkBarHeights } from '#/data/selectors'
import type { SparkPoint } from '#/data/mockData'

interface SparkBarsProps {
  points: SparkPoint[]
}

export function SparkBars({ points }: SparkBarsProps) {
  const bars = sparkBarHeights(points)
  return (
    <div className="flex items-end gap-[7px]" style={{ height: 78 }}>
      {bars.map((bar) => (
        <div
          key={bar.label}
          className="flex flex-1 flex-col items-center gap-[5px]"
        >
          <div
            className="w-full rounded-[3px]"
            style={{ background: '#5d5294', minHeight: 4, height: bar.height }}
          />
          <span
            className="mono"
            style={{ fontSize: 9, color: 'rgba(233,233,237,.4)' }}
          >
            {bar.label}
          </span>
        </div>
      ))}
    </div>
  )
}
