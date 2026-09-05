import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { AccountAvatar } from '#/components/AccountAvatar'
import { Mascot } from '#/components/Mascot'
import { PageHeader } from '#/components/PageHeader'
import { useAppStore } from '#/data/store'

export const Route = createFileRoute('/_app/settings')({
  component: SettingsPage,
})

function SettingsPage() {
  const store = useAppStore()
  const navigate = useNavigate()

  return (
    <>
      <PageHeader kicker="configuration" title="Settings" />
      <div
        className="grid min-w-0 flex-1 grid-cols-1 content-start items-start gap-[18px] lg:grid-cols-2"
        style={{
          padding: '20px 24px 8px',
          maxWidth: 1000,
        }}
      >
        <div
          className="card elev-sm"
          style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
        >
          <span className="card-kicker mono">payday cycle</span>
          <div className="flex flex-col gap-[11px] sm:flex-row">
            <div className="field sm:flex-1">
              <label>First payday (anchor)</label>
              <input className="input mono" defaultValue="14/11/2025" />
            </div>
            <div className="field sm:w-[132px]">
              <label>Deposit day</label>
              <input className="input mono" defaultValue="Friday" />
            </div>
          </div>
          <div className="field">
            <label>Pay cadence</label>
            <div className="seg w-full">
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="cadence" />
                Weekly
              </label>
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="cadence" defaultChecked />
                Bi-weekly
              </label>
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="cadence" />
                Semi-monthly
              </label>
            </div>
          </div>
          <div className="field">
            <label>Cycle boundary</label>
            <div className="flex flex-col gap-[6px]">
              <label className="radio">
                <input type="radio" name="bound" defaultChecked />
                <span className="dot" />
                <span>A cycle runs payday → day before next payday</span>
              </label>
              <label className="radio">
                <input type="radio" name="bound" />
                <span className="dot" />
                <span>A cycle runs 1st → 15th, 16th → end of month</span>
              </label>
            </div>
          </div>
          <div className="field">
            <label>Cycle name pattern</label>
            <input className="input mono" defaultValue="{Month} P{n}" />
          </div>
          <div
            className="mono"
            style={{ fontSize: 11, color: 'rgba(233,233,237,.42)' }}
          >
            Next cycles: December P1 · December P2 · January P1
          </div>
        </div>

        <div
          className="card elev-sm"
          style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
        >
          <span className="card-kicker mono">how bills land in a cycle</span>
          <div className="field">
            <label>Pull a bill into the cycle when…</label>
            <div className="flex flex-col gap-[6px]">
              <label className="radio">
                <input type="radio" name="pull" defaultChecked />
                <span className="dot" />
                <span>its due day falls before the next payday</span>
              </label>
              <label className="radio">
                <input type="radio" name="pull" />
                <span className="dot" />
                <span>its due day falls inside the cycle dates</span>
              </label>
            </div>
          </div>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Auto-pay bills start marked paid</span>
          </label>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Carry the previous cycle&apos;s net into bank balance</span>
          </label>
          <label className="radio">
            <input type="checkbox" />
            <span className="dot" />
            <span>Flag three-paycheck cycles</span>
          </label>
          <div
            className="h-px"
            style={{ background: 'rgba(233,233,237,.1)' }}
          />
          <div className="flex flex-col gap-[11px] sm:flex-row">
            <div className="field sm:flex-1">
              <label>Opening bank balance</label>
              <input className="input mono" defaultValue="$3,412.80" />
            </div>
            <div className="field sm:w-[120px]">
              <label>Buffer to keep</label>
              <input className="input mono" defaultValue="$250.00" />
            </div>
          </div>
        </div>

        <div
          className="card elev-sm"
          style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
        >
          <span className="card-kicker mono">pto defaults</span>
          <div className="flex flex-col gap-[11px] sm:flex-row">
            <div className="field sm:flex-1">
              <label>Hours available per year</label>
              <input className="input mono" defaultValue="200.00" />
            </div>
            <div className="field sm:w-[120px]">
              <label>Holiday default</label>
              <input className="input mono" defaultValue="8.00" />
            </div>
          </div>
          <div className="field">
            <label>Accrual</label>
            <div className="seg w-full">
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="accrual" defaultChecked />
                Granted up front
              </label>
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="accrual" />
                Per paycheck
              </label>
            </div>
          </div>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Copy holiday hours forward each new year</span>
          </label>
        </div>

        <div
          className="card elev-sm"
          style={{ background: '#1b1d2e', padding: '18px 20px', gap: 14 }}
        >
          <span className="card-kicker mono">display &amp; account</span>
          <div className="field">
            <label>Date format</label>
            <div className="seg w-full">
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="datefmt" defaultChecked />
                DD/MM/YYYY
              </label>
              <label className="seg-opt flex-1 justify-center">
                <input type="radio" name="datefmt" />
                MM/DD/YYYY
              </label>
            </div>
          </div>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Terminal keybindings (h i b l p s)</span>
          </label>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Let bae comment on the cycle</span>
          </label>
          <div
            className="h-px"
            style={{ background: 'rgba(233,233,237,.1)' }}
          />
          <div className="flex flex-col items-start gap-[11px] sm:flex-row sm:items-center">
            <div className="flex items-center gap-[11px]">
              <AccountAvatar size={40} />
              <div>
                <div className="mono" style={{ fontSize: 13 }}>
                  tj@money.bae
                </div>
                <div
                  className="mono"
                  style={{ fontSize: 11, color: 'rgba(233,233,237,.45)' }}
                >
                  signed in on this machine
                </div>
              </div>
            </div>
            <button
              className="btn btn-primary mono sm:ml-auto"
              style={{ fontSize: 12 }}
              onClick={store.signOut}
            >
              Log out
            </button>
          </div>
        </div>
      </div>

      {/* Nothing on this page is wired to real settings yet — this stays up
          permanently (no close button) until that work happens. */}
      <div
        className="fixed inset-0 z-50 grid place-items-center p-4"
        style={{ background: 'rgba(16,17,32,.72)' }}
      >
        <div
          className="card elev-lg flex flex-col items-center"
          style={{
            background: '#1b1d2e',
            border: '1px solid rgba(233,233,237,.14)',
            padding: '28px 36px',
            gap: 14,
          }}
        >
          <Mascot size={96} />
          <span
            className="mono"
            style={{
              fontSize: 15,
              letterSpacing: '.08em',
              color: '#e9e9ed',
              textAlign: 'center',
            }}
          >
            UNDER CONSTRUCTION
          </span>
          <span
            className="mono"
            style={{
              fontSize: 12,
              color: 'rgba(233,233,237,.5)',
              textAlign: 'center',
            }}
          >
            bae is still wiring this page up. kk?
          </span>
          <button
            className="btn btn-secondary mono"
            style={{ fontSize: 12 }}
            onClick={() => navigate({ to: '/' })}
          >
            Go home
          </button>
        </div>
      </div>
    </>
  )
}
