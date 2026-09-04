import { sparkBarHeights } from '#/data/selectors'
import type { SparkPoint } from '#/data/mockData'

interface SparkBarsProps {
  points: SparkPoint[]
}

// Diverging chart: positive bars grow up from the label row, negative bars
// grow down from it, so the labels sit centered top-to-bottom in the box
// and the whole thing stretches to fill whatever height its card gives it.
export function SparkBars({ points }: SparkBarsProps) {
  const bars = sparkBarHeights(points)
  return (
    <div className="flex flex-1" style={{ gap: 7, minHeight: 0 }}>
      {bars.map((bar) => (
        <div
          key={bar.label}
          className="flex flex-1 flex-col items-center"
          style={{ minHeight: 0 }}
        >
          <div
            className="flex w-full flex-1 items-end justify-center"
            style={{ minHeight: 0 }}
          >
            {bar.positiveHeightPct > 0 && (
              <div
                className="w-full rounded-t-[3px]"
                style={{
                  background: '#5d5294',
                  minHeight: 3,
                  height: `${bar.positiveHeightPct}%`,
                }}
              />
            )}
          </div>
          <span
            className="mono"
            style={{
              fontSize: 9,
              color: 'rgba(233,233,237,.4)',
              padding: '4px 0',
              flex: 'none',
            }}
          >
            {bar.label}
          </span>
          <div
            className="flex w-full flex-1 items-start justify-center"
            style={{ minHeight: 0 }}
          >
            {bar.negativeHeightPct > 0 && (
              <div
                className="w-full rounded-b-[3px]"
                style={{
                  background: '#c85a5a',
                  minHeight: 3,
                  height: `${bar.negativeHeightPct}%`,
                }}
              />
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
