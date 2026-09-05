import type { ReactNode } from 'react'

interface PageHeaderProps {
  kicker: string
  title: string
  actions?: ReactNode
}

export function PageHeader({ kicker, title, actions }: PageHeaderProps) {
  return (
    <>
      <header
        className="flex items-end gap-[14px]"
        style={{ padding: '18px 24px 14px' }}
      >
        <div>
          <div
            className="mono"
            style={{
              fontSize: 10.5,
              letterSpacing: '.2em',
              textTransform: 'uppercase',
              color: 'rgba(233,233,237,.4)',
            }}
          >
            {kicker}
          </div>
          <h3 style={{ margin: '3px 0 0' }}>{title}</h3>
        </div>
        {actions && <div className="ml-auto flex gap-[8px]">{actions}</div>}
      </header>
      <div
        className="h-px"
        style={{
          background:
            'linear-gradient(to right,transparent,rgba(233,233,237,.16) 48px,rgba(233,233,237,.16) calc(100% - 48px),transparent)',
        }}
      />
    </>
  )
}
