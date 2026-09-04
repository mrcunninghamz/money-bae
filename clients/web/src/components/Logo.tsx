interface LogoProps {
  size?: number
  className?: string
}

export function Logo({ size = 36, className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 110 100"
      style={{ width: size, flex: 'none' }}
      className={className}
      aria-label="money-bae"
    >
      <rect
        x="20"
        y="52"
        width="70"
        height="26"
        rx="7"
        fill="#2b2741"
        stroke="#796cbf"
        strokeWidth="3"
        transform="rotate(-5 55 65)"
      />
      <path
        d="M28 40 L31 18 L46 34 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <path
        d="M78 40 L75 18 L60 34 Z"
        fill="#2b2741"
        stroke="#9184d9"
        strokeWidth="3"
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
        strokeWidth="3"
      />
      <g fill="#e9e9ed">
        <ellipse className="eye" cx="38" cy="48" rx="3.6" ry="4.2" />
        <ellipse className="eye" cx="68" cy="48" rx="3.6" ry="4.2" />
      </g>
      <path
        d="M49 55 q4 4.5 8 0"
        fill="none"
        stroke="#e9e9ed"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
    </svg>
  )
}
