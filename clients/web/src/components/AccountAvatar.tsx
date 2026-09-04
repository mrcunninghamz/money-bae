interface AccountAvatarProps {
  size?: number
  animate?: boolean
  className?: string
}

export function AccountAvatar({
  size = 40,
  animate = false,
  className,
}: AccountAvatarProps) {
  return (
    <svg
      viewBox="0 0 120 106"
      style={{ width: size, flex: 'none' }}
      className={[animate ? 'bob' : '', className].filter(Boolean).join(' ')}
      aria-label="account"
    >
      <path
        d="M14 104 C20 82 38 76 58 76 C78 76 96 82 102 104"
        fill="#1b1d2e"
        stroke="#796cbf"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <circle
        cx="20"
        cy="50"
        r="5.5"
        fill="#2b2741"
        stroke="#796cbf"
        strokeWidth="2.6"
      />
      <circle
        cx="96"
        cy="50"
        r="5.5"
        fill="#2b2741"
        stroke="#796cbf"
        strokeWidth="2.6"
      />
      <rect
        x="22"
        y="26"
        width="72"
        height="44"
        rx="12"
        fill="#161826"
        stroke="#b5abfc"
        strokeWidth="3"
      />
      <path d="M24 30 C30 10 86 10 92 30 C82 20 34 20 24 30 Z" fill="#5d5294" />
      <g fill="#e9e9ed">
        <ellipse className="eye" cx="43" cy="45" rx="4.8" ry="5.4" />
        <ellipse className="eye" cx="73" cy="45" rx="4.8" ry="5.4" />
      </g>
      <g fill="#161826">
        <circle cx="44.4" cy="43" r="1.6" />
        <circle cx="74.4" cy="43" r="1.6" />
      </g>
      <path
        d="M52 55 q6 6 12 0"
        fill="none"
        stroke="#e9e9ed"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
    </svg>
  )
}
