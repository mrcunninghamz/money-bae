import { useEffect, useState } from 'react'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { Mascot } from '#/components/Mascot'
import { PageHeader } from '#/components/PageHeader'
import { SparkBars } from '#/components/SparkBars'
import type { Bill, CurrentLedger, LedgerHistoryEntry, Pto } from '#/data/api'
import {
  getCurrentLedger,
  getCurrentPto,
  getLedgerHistory,
  moneyToNumber,
} from '#/data/api'
import { formatCurrency, formatDateMMDDYYYY } from '#/data/format'
import type { SparkPoint } from '#/data/mockData'
import { useAppStore } from '#/data/store'

export const Route = createFileRoute('/_app/')({
  component: HomePage,
})

function checkInMessage(
  status: CurrentLedger['checkIn']['status'],
  net: number,
): string {
  switch (status) {
    case 'good':
      return `looking good. ${formatCurrency(net)} free after everything.`
    case 'tight':
      return `cutting it close. ${formatCurrency(net)} free after everything.`
    case 'negative':
      return `uh oh. ${formatCurrency(Math.abs(net))} in the hole after everything.`
  }
}

function dueDayLabel(day: number): string {
  if (day >= 31) return 'month end'
  if (day % 10 === 1 && day !== 11) return `${day}st`
  if (day % 10 === 2 && day !== 12) return `${day}nd`
  if (day % 10 === 3 && day !== 13) return `${day}rd`
  return `${day}th`
}

const NEXT_UP_LIMIT = 5

// Bills due later this month, from today's day-of-month through the 31st —
// bills already due earlier this month, or due next month, don't show here.
// Capped to the soonest NEXT_UP_LIMIT so the card doesn't grow unbounded.
function upcomingBills(bills: Bill[], today: Date): Bill[] {
  const todayDay = today.getDate()
  return bills
    .filter((b) => b.dueDay != null && b.dueDay >= todayDay)
    .sort((a, b) => (a.dueDay ?? 0) - (b.dueDay ?? 0))
    .slice(0, NEXT_UP_LIMIT)
}

function historyToSparkPoints(history: LedgerHistoryEntry[]): SparkPoint[] {
  return history.map((entry) => ({
    label: new Date(entry.date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      timeZone: 'UTC',
    }),
    value: Number(entry.net),
  }))
}

function HomePage() {
  const store = useAppStore()
  const navigate = useNavigate()
  const [currentLedger, setCurrentLedger] = useState<CurrentLedger | null>(null)
  const [history, setHistory] = useState<LedgerHistoryEntry[]>([])
  const [currentYearPto, setCurrentYearPto] = useState<Pto | null>(null)

  useEffect(() => {
    void store.ensureBills()
    getCurrentLedger()
      .then(setCurrentLedger)
      .catch((err: unknown) => {
        console.error('failed to load current ledger', err)
      })
    getLedgerHistory()
      .then(setHistory)
      .catch((err: unknown) => {
        console.error('failed to load ledger history', err)
      })
    getCurrentPto()
      .then(setCurrentYearPto)
      .catch((err: unknown) => {
        console.error('failed to load current pto', err)
      })
  }, [])

  const availableFunds = currentLedger
    ? Number(currentLedger.availableFunds)
    : 0
  const paid = currentLedger ? Number(currentLedger.paid) : 0
  const planned = currentLedger ? Number(currentLedger.planned) : 0
  const net = currentLedger ? Number(currentLedger.net) : 0
  const paidPct = availableFunds
    ? Math.min(100, Math.max(0, (paid / availableFunds) * 100))
    : 0
  const plannedPct = availableFunds
    ? Math.min(100, Math.max(0, (planned / availableFunds) * 100))
    : 0
  const nextUp = upcomingBills(store.bills, new Date())

  return (
    <>
      <PageHeader
        kicker="current cycle"
        title={
          currentLedger
            ? `${currentLedger.name ?? 'Current cycle'} · ${formatDateMMDDYYYY(currentLedger.date)}`
            : 'Current cycle'
        }
      />
      <div
        className="flex min-w-0 flex-1 flex-col gap-[18px]"
        style={{ padding: '20px 24px 8px' }}
      >
        <div className="grid grid-cols-1 gap-[18px] lg:grid-cols-[1.35fr_1fr]">
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
          >
            <div className="flex items-baseline gap-[10px]">
              <span className="card-kicker mono">current cycle</span>
            </div>
            <div className="flex flex-wrap items-end gap-[26px]">
              <div>
                <div
                  className="mono"
                  style={{ fontSize: 11, color: 'rgba(233,233,237,.5)' }}
                >
                  available funds
                </div>
                <div
                  className="mono"
                  style={{
                    fontSize: 38,
                    lineHeight: 1.05,
                    letterSpacing: '-.02em',
                  }}
                >
                  {formatCurrency(availableFunds)}
                </div>
              </div>
              <div>
                <div
                  className="mono"
                  style={{ fontSize: 11, color: 'rgba(233,233,237,.5)' }}
                >
                  after all bills
                </div>
                <div
                  className="mono"
                  style={{ fontSize: 26, lineHeight: 1.1, color: '#b5abfc' }}
                >
                  {formatCurrency(net)}
                </div>
              </div>
            </div>
            <div
              className="flex overflow-hidden rounded-[5px]"
              style={{ height: 10, background: '#292b31' }}
            >
              <div style={{ width: `${paidPct}%`, background: '#796cbf' }} />
              <div style={{ width: `${plannedPct}%`, background: '#423a6a' }} />
            </div>
            <div
              className="mono flex flex-wrap gap-[16px]"
              style={{ fontSize: 11, color: 'rgba(233,233,237,.5)' }}
            >
              <span>paid {formatCurrency(paid)}</span>
              <span>planned {formatCurrency(planned)}</span>
              <span>free {formatCurrency(net)}</span>
            </div>
          </div>

          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '18px 20px', gap: 12 }}
          >
            <span className="card-kicker mono">bae check-in</span>
            <div className="flex items-center gap-[14px]">
              <Mascot size={56} />
              <div>
                <div style={{ fontSize: 15, lineHeight: 1.4 }}>
                  {currentLedger
                    ? checkInMessage(currentLedger.checkIn.status, net)
                    : 'ni howdy!'}
                </div>
                <div
                  className="mono"
                  style={{
                    fontSize: 12,
                    color: 'rgba(233,233,237,.5)',
                    marginTop: 6,
                  }}
                >
                  {currentLedger
                    ? `${currentLedger.unpaidCount} bill${currentLedger.unpaidCount === 1 ? '' : 's'} unpaid · ${formatCurrency(planned)} still planned`
                    : 'add a ledger cycle to see your check-in.'}
                </div>
              </div>
            </div>
            <div
              className="flex items-center gap-[14px]"
              style={{ marginTop: 2 }}
            >
              <button
                className="btn btn-secondary mono"
                style={{ fontSize: 12 }}
                disabled={!currentLedger}
                onClick={() =>
                  currentLedger &&
                  navigate({
                    to: '/ledger/$periodId',
                    params: { periodId: currentLedger.id },
                  })
                }
              >
                Open cycle
              </button>
              <button
                className="btn btn-ghost mono"
                style={{ fontSize: 12, color: '#9184d9' }}
                onClick={() =>
                  currentYearPto
                    ? navigate({
                        to: '/pto/$year',
                        params: { year: String(currentYearPto.year) },
                      })
                    : navigate({ to: '/pto' })
                }
              >
                {currentYearPto
                  ? `PTO ${Math.round(Number(currentYearPto.hoursRemaining))}h left`
                  : 'Setup PTO'}
              </button>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-[18px] lg:grid-cols-3">
          <div
            className="card elev-sm min-h-[200px] lg:min-h-0"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 10 }}
          >
            <span className="card-kicker mono">net by cycle</span>
            <SparkBars points={historyToSparkPoints(history)} />
          </div>
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 8 }}
          >
            <span className="card-kicker mono">next up</span>
            {nextUp.length === 0 && (
              <div
                className="mono"
                style={{ fontSize: 12, color: 'rgba(233,233,237,.4)' }}
              >
                nothing due the rest of this month
              </div>
            )}
            {nextUp.map((bill) => (
              <div
                key={bill.id}
                className="flex items-baseline gap-[10px]"
                style={{ fontSize: 13 }}
              >
                <span
                  className="mono"
                  style={{
                    color: 'rgba(233,233,237,.45)',
                    width: 62,
                    whiteSpace: 'nowrap',
                  }}
                >
                  {dueDayLabel(bill.dueDay ?? 0)}
                </span>
                <span className="flex-1">{bill.name}</span>
                <span className="mono">
                  {formatCurrency(moneyToNumber(bill.amount))}
                </span>
              </div>
            ))}
          </div>
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 8 }}
          >
            <span className="card-kicker mono">shortcuts</span>
            <div
              className="mono"
              style={{
                fontSize: 12.5,
                lineHeight: 1.9,
                color: 'rgba(233,233,237,.7)',
              }}
            >
              <div>
                <span style={{ color: '#9184d9' }}>h</span> home &nbsp;{' '}
                <span style={{ color: '#9184d9' }}>i</span> income &nbsp;{' '}
                <span style={{ color: '#9184d9' }}>b</span> bills
              </div>
              <div>
                <span style={{ color: '#9184d9' }}>l</span> ledger &nbsp;{' '}
                <span style={{ color: '#9184d9' }}>p</span> pto &nbsp;{' '}
                <span style={{ color: '#9184d9' }}>/</span> search
              </div>
              <div>
                <span style={{ color: '#9184d9' }}>space</span> toggle paid
                &nbsp; <span style={{ color: '#9184d9' }}>e</span> edit
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
