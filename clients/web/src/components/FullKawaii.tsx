interface FullKawaiiProps {
  size?: number
  className?: string
}

export function FullKawaii({ size = 86, className }: FullKawaiiProps) {
  return (
    <svg
      viewBox="0 0 120 100"
      style={{ width: size, flex: 'none' }}
      className={['bob', className].filter(Boolean).join(' ')}
      aria-label="money-bae kawaii mark"
    >
      <rect
        x="26"
        y="52"
        width="76"
        height="28"
        rx="8"
        fill="#2b2741"
        stroke="#5d5294"
        strokeWidth="2.5"
        transform="rotate(4 64 66)"
      />
      <path
        d="M32 34 L30 12 L48 28 Z"
        fill="#423a6a"
        stroke="#b5abfc"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <path
        d="M84 34 L86 12 L68 28 Z"
        fill="#423a6a"
        stroke="#b5abfc"
        strokeWidth="2.5"
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
        strokeWidth="2.5"
      />
      <g stroke="#796cbf" strokeWidth="2" strokeLinecap="round">
        <path d="M20 44 L4 38" />
        <path d="M20 51 L3 51" />
        <path d="M20 58 L5 64" />
        <path d="M96 44 L112 38" />
        <path d="M96 51 L113 51" />
        <path d="M96 58 L111 64" />
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
      <text
        x="58"
        y="68"
        textAnchor="middle"
        fontFamily="ui-monospace, Menlo, monospace"
        fontSize="10"
        fill="#796cbf"
      >
        $ $
      </text>
    </svg>
  )
}
