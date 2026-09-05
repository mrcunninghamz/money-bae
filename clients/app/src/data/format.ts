export function formatCurrency(amount: number): string {
  const sign = amount < 0 ? '-$' : '$'
  return (
    sign +
    Math.abs(amount).toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  )
}

export function formatHours(hours: number): string {
  return hours.toFixed(2)
}

// MM/DD/YYYY, matching the label on the date inputs in EditModal.
export function formatDateMMDDYYYY(iso: string): string {
  const d = new Date(iso)
  const mm = String(d.getUTCMonth() + 1).padStart(2, '0')
  const dd = String(d.getUTCDate()).padStart(2, '0')
  const yyyy = d.getUTCFullYear()
  return `${mm}/${dd}/${yyyy}`
}

export function parseDateMMDDYYYY(value: string): string {
  const [mm, dd, yyyy] = value.split('/').map(Number)
  return new Date(Date.UTC(yyyy, mm - 1, dd)).toISOString()
}

// YYYY-MM-DD, the value format native <input type="date"> requires.
export function formatDateInputValue(iso: string): string {
  const d = new Date(iso)
  const mm = String(d.getUTCMonth() + 1).padStart(2, '0')
  const dd = String(d.getUTCDate()).padStart(2, '0')
  return `${d.getUTCFullYear()}-${mm}-${dd}`
}

export function parseDateInputValue(value: string): string {
  const [yyyy, mm, dd] = value.split('-').map(Number)
  return new Date(Date.UTC(yyyy, mm - 1, dd)).toISOString()
}
