interface CoinBadgeProps {
  size?: number
  className?: string
}

export function CoinBadge({ size = 86, className }: CoinBadgeProps) {
  return (
    <svg
      viewBox="0 0 110 110"
      style={{ width: size, flex: 'none' }}
      className={['bob', className].filter(Boolean).join(' ')}
      aria-label="money-bae badge mark"
    >
      <circle
        cx="55"
        cy="58"
        r="40"
        fill="#1c1e30"
        stroke="#5d5294"
        strokeWidth="2.5"
      />
      <circle
        cx="55"
        cy="58"
        r="33"
        fill="none"
        stroke="#423a6a"
        strokeWidth="1.5"
        strokeDasharray="3 4"
      />
      <path
        d="M36 26 L34 8 L50 22 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <path
        d="M74 26 L76 8 L60 22 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <rect
        x="30"
        y="58"
        width="50"
        height="20"
        rx="5"
        fill="#2b2741"
        stroke="#796cbf"
        strokeWidth="2"
        transform="rotate(-6 55 68)"
      />
      <rect
        x="28"
        y="46"
        width="54"
        height="22"
        rx="6"
        fill="#161826"
        stroke="#b5abfc"
        strokeWidth="2.2"
      />
      <g fill="#e9e9ed">
        <ellipse className="eye" cx="43" cy="55" rx="2.8" ry="3.4" />
        <ellipse className="eye" cx="67" cy="55" rx="2.8" ry="3.4" />
      </g>
      <path
        d="M52 60 q3 3.5 6 0"
        fill="none"
        stroke="#e9e9ed"
        strokeWidth="2"
        strokeLinecap="round"
      />
    </svg>
  )
}
