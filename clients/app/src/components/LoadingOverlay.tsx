import { useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { BillStack } from '#/components/BillStack'
import { CoinBadge } from '#/components/CoinBadge'
import { FullKawaii } from '#/components/FullKawaii'
import { getLoadingSnapshot, subscribeLoading } from '#/data/api'

const ICONS = [FullKawaii, CoinBadge, BillStack]

export function LoadingOverlay() {
  const loading = useSyncExternalStore(subscribeLoading, getLoadingSnapshot)
  const [Icon, setIcon] = useState(() => ICONS[0])
  const wasLoading = useRef(false)

  useEffect(() => {
    if (loading && !wasLoading.current) {
      setIcon(() => ICONS[Math.floor(Math.random() * ICONS.length)])
    }
    wasLoading.current = loading
  }, [loading])

  if (!loading) return null

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center"
      style={{ background: 'rgba(16,17,32,.62)' }}
    >
      <div className="flex flex-col items-center gap-[10px]">
        <Icon size={80} />
        <span className="mono" style={{ color: '#e9e9ed', fontSize: 13 }}>
          kk.. bae bae!
        </span>
      </div>
    </div>
  )
}
