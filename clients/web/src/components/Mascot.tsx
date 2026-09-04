interface MascotProps {
  size?: number
  className?: string
}

export function Mascot({ size = 86, className }: MascotProps) {
  return (
    <svg
      viewBox="0 0 120 100"
      style={{ width: size, flex: 'none' }}
      className={['bob', className].filter(Boolean).join(' ')}
      aria-label="bae"
    >
      <path
        d="M32 34 L30 12 L48 28 Z"
        fill="#423a6a"
        stroke="#b5abfc"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <path
        d="M84 34 L86 12 L68 28 Z"
        fill="#423a6a"
        stroke="#b5abfc"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <rect
        x="22"
        y="28"
        width="72"
        height="44"
        rx="12"
        fill="#161826"
        stroke="#b5abfc"
        strokeWidth="3"
      />
      <g stroke="#796cbf" strokeWidth="2.4" strokeLinecap="round">
        <path d="M20 44 L6 39" />
        <path d="M20 52 L5 52" />
        <path d="M96 44 L110 39" />
        <path d="M96 52 L111 52" />
      </g>
      <g fill="#e9e9ed">
        <ellipse className="eye" cx="43" cy="47" rx="5" ry="5.6" />
        <ellipse className="eye" cx="73" cy="47" rx="5" ry="5.6" />
      </g>
      <g fill="#161826">
        <circle cx="44.6" cy="45" r="1.7" />
        <circle cx="74.6" cy="45" r="1.7" />
      </g>
      <g fill="#9184d9" opacity=".5">
        <ellipse cx="33" cy="57" rx="5" ry="2.6" />
        <ellipse cx="83" cy="57" rx="5" ry="2.6" />
      </g>
      <path
        d="M54 55 q4 4 6 0 q2 4 6 0"
        fill="none"
        stroke="#e9e9ed"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
    </svg>
  )
}
