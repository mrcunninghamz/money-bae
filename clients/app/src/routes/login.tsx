import { useEffect, useState } from 'react'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { AccountAvatar } from '#/components/AccountAvatar'
import { MASCOT_ICONS } from '#/components/mascotIcons'
import { useAppStore } from '#/data/store'

export const Route = createFileRoute('/login')({
  component: LoginPage,
})

function LoginPage() {
  const store = useAppStore()
  const navigate = useNavigate()
  const [Icon] = useState(
    () => MASCOT_ICONS[Math.floor(Math.random() * MASCOT_ICONS.length)],
  )

  useEffect(() => {
    if (store.authed) navigate({ to: '/' })
  }, [store.authed, navigate])

  if (store.authed) return null

  return (
    <div
      className="grid p-10"
      style={{
        minHeight: '100vh',
        placeItems: 'center',
        background: '#161826',
      }}
    >
      <div
        className="flex flex-col items-center gap-[28px]"
        style={{ width: 'min(400px, 100%)' }}
      >
        <div className="flex flex-col items-center gap-[10px]">
          <Icon size={100} />
          <div className="mono" style={{ fontSize: 22 }}>
            money<span style={{ color: '#9184d9' }}>·</span>bae
          </div>
          <h4 style={{ margin: '4px 0 0', fontSize: 18, textAlign: 'center' }}>
            This is not a budgeting app.
          </h4>
          <div
            style={{
              fontSize: 13,
              lineHeight: 1.4,
              color: 'rgba(233,233,237,.6)',
              textAlign: 'center',
              maxWidth: 280,
            }}
          >
            You don&apos;t need a budget, just make sure you can pay your bills!
          </div>
        </div>
        <div
          className="card elev-sm flex flex-col items-center gap-[10px]"
          style={{ background: '#1b1d2e', padding: '16px 20px', width: '100%' }}
        >
          <button
            className="btn btn-primary btn-block mono"
            type="button"
            style={{ minHeight: 40, gap: 10 }}
            onClick={store.signIn}
          >
            <AccountAvatar size={22} />
            c&apos;mon in, it&apos;s warm
          </button>
          <div
            style={{
              fontSize: 11,
              color: 'rgba(233,233,237,.4)',
              textAlign: 'center',
            }}
          >
            For the teenagers, young adults, and adults who probably need an
            adult.
          </div>
        </div>
      </div>
    </div>
  )
}
