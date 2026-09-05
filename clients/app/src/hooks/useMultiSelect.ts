import { useState } from 'react'

export function useMultiSelect() {
  const [selected, setSelected] = useState<Set<string>>(new Set())

  function toggle(id: string) {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  function isSelected(id: string) {
    return selected.has(id)
  }

  function clear() {
    setSelected(new Set())
  }

  return {
    selectedIds: Array.from(selected),
    count: selected.size,
    isSelected,
    toggle,
    clear,
  }
}
