import { useEffect } from 'react'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { AccountAvatar } from '#/components/AccountAvatar'
import { useAppStore } from '#/data/store'

export const Route = createFileRoute('/login')({
  component: LoginPage,
})

function LoginPage() {
  const store = useAppStore()
  const navigate = useNavigate()

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
      <div className="flex flex-col gap-[18px]" style={{ width: 400 }}>
        <div className="flex flex-col items-center gap-[12px]">
          <AccountAvatar size={104} animate />
          <div className="mono" style={{ fontSize: 22 }}>
            money<span style={{ color: '#9184d9' }}>·</span>bae
          </div>
          <div
            className="mono"
            style={{ fontSize: 11.5, color: 'rgba(233,233,237,.45)' }}
          >
            sign in to your cycles
          </div>
        </div>
        <form
          className="card elev-sm flex flex-col gap-[12px]"
          style={{ background: '#1b1d2e', padding: '18px 20px' }}
          onSubmit={(event) => {
            event.preventDefault()
            store.signIn()
          }}
        >
          <div className="field">
            <label>Email</label>
            <input className="input mono" defaultValue="tj@money.bae" />
          </div>
          <div className="field">
            <label>Password</label>
            <input
              className="input mono"
              type="password"
              defaultValue="hunter22"
            />
          </div>
          <label className="radio">
            <input type="checkbox" defaultChecked />
            <span className="dot" />
            <span>Keep me signed in on this machine</span>
          </label>
          <button
            className="btn btn-primary btn-block mono"
            type="submit"
            style={{ minHeight: 40 }}
          >
            Sign in
          </button>
          <div
            className="mono"
            style={{
              fontSize: 11,
              color: 'rgba(233,233,237,.4)',
              textAlign: 'center',
            }}
          >
            single-user install · local database
          </div>
        </form>
      </div>
    </div>
  )
}
