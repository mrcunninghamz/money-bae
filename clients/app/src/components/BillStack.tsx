interface BillStackProps {
  size?: number
  className?: string
}

export function BillStack({ size = 86, className }: BillStackProps) {
  return (
    <svg
      viewBox="0 0 110 100"
      style={{ width: size, flex: 'none' }}
      className={['bob', className].filter(Boolean).join(' ')}
      aria-label="money-bae mark"
    >
      <g className="tail" style={{ transformOrigin: '86px 74px' }}>
        <path
          d="M84 72 C100 70 104 56 96 48"
          fill="none"
          stroke="#796cbf"
          strokeWidth="3.5"
          strokeLinecap="round"
        />
      </g>
      <rect
        x="20"
        y="52"
        width="70"
        height="26"
        rx="7"
        fill="#2b2741"
        stroke="#796cbf"
        strokeWidth="2.5"
        transform="rotate(-5 55 65)"
      />
      <path
        d="M28 40 L31 18 L46 34 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <path
        d="M78 40 L75 18 L60 34 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <rect
        x="16"
        y="34"
        width="74"
        height="34"
        rx="9"
        fill="#161826"
        stroke="#9184d9"
        strokeWidth="2.5"
      />
      <g stroke="#5d5294" strokeWidth="2" strokeLinecap="round">
        <path d="M14 46 L2 42" />
        <path d="M14 52 L1 53" />
        <path d="M92 46 L104 42" />
        <path d="M92 52 L105 53" />
      </g>
      <g fill="#e9e9ed">
        <ellipse className="eye" cx="38" cy="48" rx="3.4" ry="4" />
        <ellipse className="eye" cx="68" cy="48" rx="3.4" ry="4" />
      </g>
      <path
        d="M49 55 q4 4.5 8 0"
        fill="none"
        stroke="#e9e9ed"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
      <text
        x="53"
        y="66"
        textAnchor="middle"
        fontFamily="ui-monospace, Menlo, monospace"
        fontSize="9"
        fill="#b5abfc"
      >
        $
      </text>
    </svg>
  )
}
