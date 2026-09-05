interface SelectCheckboxProps {
  checked: boolean
  onToggle: () => void
}

// Wraps a visually-hidden native checkbox with the app's `.checkbox` styling
// (see components.css), matching the `.radio` pattern used elsewhere —
// never the unstyled native control. stopPropagation so this can sit inside
// a row that itself has its own onClick (e.g. toggling the same selection)
// without double-toggling.
export function SelectCheckbox({ checked, onToggle }: SelectCheckboxProps) {
  return (
    <label className="checkbox" onClick={(e) => e.stopPropagation()}>
      <input type="checkbox" checked={checked} onChange={onToggle} />
      <span className="box" />
    </label>
  )
}
