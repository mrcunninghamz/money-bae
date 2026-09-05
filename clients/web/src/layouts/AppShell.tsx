import { useEffect } from 'react'
import {
  Link,
  Outlet,
  useNavigate,
  useRouterState,
} from '@tanstack/react-router'
import { AccountAvatar } from '#/components/AccountAvatar'
import { ConfirmDeleteDialog } from '#/components/ConfirmDeleteDialog'
import { EditModal } from '#/components/EditModal'
import { LoadingOverlay } from '#/components/LoadingOverlay'
import { Logo } from '#/components/Logo'
import { Mascot } from '#/components/Mascot'
import { ToastStack } from '#/components/ToastStack'
import { useAppStore } from '#/data/store'

const NAV_ITEMS = [
  { key: 'h', label: 'Home', to: '/' },
  { key: 'l', label: 'Ledger', to: '/ledger' },
  { key: 'b', label: 'Bills', to: '/bills' },
  { key: 'i', label: 'Income', to: '/income' },
  { key: 'p', label: 'PTO', to: '/pto' },
  { key: 's', label: 'Settings', to: '/settings' },
] as const

const SHORTCUT_TARGETS: Record<string, string> = {
  h: '/',
  i: '/income',
  b: '/bills',
  l: '/ledger',
  p: '/pto',
  s: '/settings',
}

function crumbFor(pathname: string): string {
  if (pathname === '/') return 'home'
  if (pathname === '/ledger') return 'ledger'
  if (pathname.startsWith('/ledger/')) return 'ledger / item'
  if (pathname === '/bills') return 'bills'
  if (pathname === '/income') return 'income'
  if (pathname === '/pto') return 'pto'
  if (pathname.startsWith('/pto/')) return 'pto / year'
  if (pathname === '/settings') return 'settings'
  return ''
}

export function AppShell() {
  const store = useAppStore()
  const navigate = useNavigate()
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })

  useEffect(() => {
    if (!store.authed) navigate({ to: '/login' })
  }, [store.authed, navigate])

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null
      if (target && /input|textarea/i.test(target.tagName)) return
      const to = SHORTCUT_TARGETS[event.key.toLowerCase()]
      if (to) navigate({ to })
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [navigate])

  if (!store.authed) return null

  return (
    <div className="flex" style={{ minHeight: '100vh', background: '#161826' }}>
      <aside
        className="flex flex-none flex-col gap-[18px]"
        style={{
          width: 212,
          padding: '16px 12px',
          borderRight: '1px solid rgba(233,233,237,.08)',
          background: '#14162399',
        }}
      >
        <div className="flex items-center gap-[9px] px-1">
          <Logo size={36} />
          <span
            className="mono"
            style={{ fontSize: 15, letterSpacing: '-.01em' }}
          >
            money<span style={{ color: '#9184d9' }}>·</span>bae
          </span>
        </div>

        <nav className="mb-nav flex flex-col gap-[3px]">
          {NAV_ITEMS.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              activeOptions={{ exact: item.to === '/' }}
              activeProps={{ 'aria-current': 'page' }}
            >
              <span
                className="mono"
                style={{ width: 14, fontSize: 11, opacity: 0.6 }}
              >
                {item.key}
              </span>
              <span>{item.label}</span>
            </Link>
          ))}
        </nav>

        <div className="card mt-auto" style={{ background: '#1b1d2e', gap: 8 }}>
          <div className="flex items-center gap-[9px]">
            <Mascot size={46} />
            <div>
              <div
                className="mono"
                style={{
                  fontSize: 10,
                  letterSpacing: '.16em',
                  textTransform: 'uppercase',
                  color: '#9184d9',
                }}
              >
                bae says
              </div>
              <div
                style={{
                  fontSize: 12.5,
                  lineHeight: 1.35,
                  color: 'rgba(233,233,237,.8)',
                }}
              >
                ni howdy!
              </div>
            </div>
          </div>
          <div
            className="mono"
            style={{ fontSize: 11, color: 'rgba(233,233,237,.45)' }}
          >
            sorry for our dust, we are doing some work!
          </div>
        </div>

        <div className="relative">
          {store.accountOpen && (
            <div
              className="card elev-md absolute z-[3] gap-[2px]"
              style={{
                bottom: 52,
                left: 0,
                right: 0,
                background: '#232532',
                padding: 6,
              }}
            >
              <button
                className="btn btn-ghost mono w-full justify-start"
                style={{ fontSize: 12.5, color: '#e9e9ed' }}
                onClick={() => {
                  store.toggleAccountMenu()
                  navigate({ to: '/settings' })
                }}
              >
                Settings
              </button>
              <button
                className="btn btn-ghost mono w-full justify-start"
                style={{ fontSize: 12.5 }}
                onClick={store.signOut}
              >
                Log out
              </button>
            </div>
          )}
          <button
            className="btn btn-secondary w-full justify-start gap-[9px]"
            style={{ padding: '7px 9px' }}
            onClick={store.toggleAccountMenu}
          >
            <AccountAvatar size={28} />
            <span
              className="flex flex-col items-start"
              style={{ lineHeight: 1.25 }}
            >
              <span className="mono" style={{ fontSize: 12.5 }}>
                {store.email}
              </span>
            </span>
          </button>
        </div>
      </aside>

      <main className="flex min-w-0 flex-1 flex-col">
        <Outlet />
        <footer
          className="mono flex gap-[18px]"
          style={{
            padding: '10px 24px',
            borderTop: '1px solid rgba(233,233,237,.08)',
            fontSize: 11.5,
            color: 'rgba(233,233,237,.42)',
          }}
        >
          <span>h home</span>
          <span>i income</span>
          <span>b bills</span>
          <span>l ledger</span>
          <span>p pto</span>
          <span>s settings</span>
          <span className="ml-auto">{crumbFor(pathname)}</span>
        </footer>
      </main>

      <EditModal />
      <ConfirmDeleteDialog />
      <LoadingOverlay />
      <ToastStack />
    </div>
  )
}
