import { useEffect, useState } from 'react'
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
  const [mobileNavOpen, setMobileNavOpen] = useState(false)

  useEffect(() => {
    if (!store.authed) navigate({ to: '/login' })
  }, [store.authed, navigate])

  useEffect(() => {
    setMobileNavOpen(false)
  }, [pathname])

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null
      if (target && /input|textarea/i.test(target.tagName)) return
      if (event.key === 'Escape' && mobileNavOpen) {
        setMobileNavOpen(false)
        return
      }
      const to = SHORTCUT_TARGETS[event.key.toLowerCase()]
      if (to) navigate({ to })
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [navigate, mobileNavOpen])

  if (!store.authed) return null

  return (
    <div className="flex" style={{ minHeight: '100vh', background: '#161826' }}>
      {mobileNavOpen && (
        <div
          className="fixed inset-0 z-40 md:hidden"
          style={{ background: 'rgba(16,17,32,.62)' }}
          onClick={() => setMobileNavOpen(false)}
        />
      )}
      <aside
        className={
          mobileNavOpen
            ? 'fixed inset-y-0 left-0 z-50 flex flex-none flex-col gap-[18px] md:static md:z-auto'
            : 'hidden flex-none flex-col gap-[18px] md:flex'
        }
        style={{
          width: 212,
          padding: '16px 12px',
          borderRight: '1px solid rgba(233,233,237,.08)',
          background: '#161826',
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
                className="mono hidden md:inline"
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
              className="flex min-w-0 flex-col items-start"
              style={{ lineHeight: 1.25 }}
            >
              <span
                className="mono"
                style={{
                  fontSize: 12.5,
                  maxWidth: '100%',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
                title={store.email ?? undefined}
              >
                {store.email}
              </span>
            </span>
          </button>
        </div>
      </aside>

      <main className="flex min-w-0 flex-1 flex-col">
        <div
          className="flex items-center gap-[10px] md:hidden"
          style={{
            padding: '12px 16px',
            borderBottom: '1px solid rgba(233,233,237,.08)',
          }}
        >
          <button
            type="button"
            className="btn btn-secondary btn-icon"
            aria-label="Open navigation"
            onClick={() => setMobileNavOpen(true)}
          >
            <svg viewBox="0 0 20 20" style={{ width: 18 }} aria-hidden="true">
              <path
                d="M3 5.5h14M3 10h14M3 14.5h14"
                stroke="currentColor"
                strokeWidth="1.6"
                strokeLinecap="round"
              />
            </svg>
          </button>
          <Logo size={26} />
          <span
            className="mono"
            style={{ fontSize: 14, letterSpacing: '-.01em' }}
          >
            money<span style={{ color: '#9184d9' }}>·</span>bae
          </span>
        </div>
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
          <span className="hidden md:inline">h home</span>
          <span className="hidden md:inline">i income</span>
          <span className="hidden md:inline">b bills</span>
          <span className="hidden md:inline">l ledger</span>
          <span className="hidden md:inline">p pto</span>
          <span className="hidden md:inline">s settings</span>
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
