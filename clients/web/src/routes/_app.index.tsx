import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { Mascot } from '#/components/Mascot'
import { PageHeader } from '#/components/PageHeader'
import { SparkBars } from '#/components/SparkBars'
import { moneyToNumber } from '#/data/api'
import { formatCurrency } from '#/data/format'
import {
  CURRENT_CYCLE_DATE,
  CURRENT_CYCLE_LABEL,
  DAYS_TO_PAYDAY,
  nextUp,
  sparkSeries,
} from '#/data/mockData'
import { useAppStore } from '#/data/store'

export const Route = createFileRoute('/_app/')({
  component: HomePage,
})

function HomePage() {
  const store = useAppStore()
  const navigate = useNavigate()
  // "Current cycle" is whichever ledger is most recent (the API returns
  // ledgers newest-first by default) — the old mock's paid/planned/unpaid
  // split doesn't have a clean equivalent without fetching that ledger's
  // bill details, so this card sticks to what the ledger itself reports.
  const latestLedger = store.ledgers.at(0)
  const firstPto = store.ptos.at(0)
  const bank = latestLedger ? moneyToNumber(latestLedger.bankBalance) : 0
  const cycleIncome = latestLedger ? moneyToNumber(latestLedger.income) : 0
  const expenses = latestLedger ? moneyToNumber(latestLedger.expenses) : 0
  const net = latestLedger ? moneyToNumber(latestLedger.net) : 0
  const availableFunds = bank + cycleIncome

  return (
    <>
      <PageHeader
        kicker="current cycle"
        title={`${CURRENT_CYCLE_LABEL} · ${CURRENT_CYCLE_DATE}`}
      />
      <div
        className="flex min-w-0 flex-1 flex-col gap-[18px]"
        style={{ padding: '20px 24px 8px' }}
      >
        <div
          className="grid gap-[18px]"
          style={{ gridTemplateColumns: '1.35fr 1fr' }}
        >
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
          >
            <div className="flex items-baseline gap-[10px]">
              <span className="card-kicker mono">current cycle</span>
              <span className="tag tag-accent mono flex-none whitespace-nowrap">
                {DAYS_TO_PAYDAY} days to payday
              </span>
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
              <div style={{ width: '54%', background: '#796cbf' }} />
              <div style={{ width: '5%', background: '#423a6a' }} />
            </div>
            <div
              className="mono flex flex-wrap gap-[16px]"
              style={{ fontSize: 11, color: 'rgba(233,233,237,.5)' }}
            >
              <span>expenses {formatCurrency(expenses)}</span>
              <span>free {formatCurrency(net)}</span>
            </div>
          </div>

          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '18px 20px', gap: 12 }}
          >
            <span className="card-kicker mono">bae check-in</span>
            <div className="flex items-center gap-[14px]">
              <Mascot size={86} />
              <div>
                <div style={{ fontSize: 14, lineHeight: 1.4 }}>ni howdy!</div>
                <div
                  className="mono"
                  style={{
                    fontSize: 11,
                    color: 'rgba(233,233,237,.45)',
                    marginTop: 5,
                  }}
                >
                  sorry for our dust, we are doing some work!
                </div>
              </div>
            </div>
            <div className="flex gap-[8px]" style={{ marginTop: 2 }}>
              <button
                className="btn btn-secondary mono"
                style={{ fontSize: 12 }}
                disabled={!latestLedger}
                onClick={() =>
                  latestLedger &&
                  navigate({
                    to: '/ledger/$periodId',
                    params: { periodId: latestLedger.id },
                  })
                }
              >
                Open cycle
              </button>
              <button
                className="btn btn-ghost mono"
                style={{ fontSize: 12 }}
                disabled={!firstPto}
                onClick={() =>
                  firstPto &&
                  navigate({
                    to: '/pto/$year',
                    params: { year: String(firstPto.year) },
                  })
                }
              >
                PTO
              </button>
            </div>
          </div>
        </div>

        <div
          className="grid gap-[18px]"
          style={{ gridTemplateColumns: '1fr 1fr 1fr' }}
        >
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 10 }}
          >
            <span className="card-kicker mono">net by cycle</span>
            <SparkBars points={sparkSeries} />
          </div>
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 8 }}
          >
            <span className="card-kicker mono">next up</span>
            {nextUp.map((entry) => (
              <div
                key={entry.name}
                className="flex items-baseline gap-[10px]"
                style={{ fontSize: 13 }}
              >
                <span
                  className="mono"
                  style={{ color: 'rgba(233,233,237,.45)', width: 44 }}
                >
                  {entry.due}
                </span>
                <span className="flex-1">{entry.name}</span>
                <span className="mono">{formatCurrency(entry.amount)}</span>
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
