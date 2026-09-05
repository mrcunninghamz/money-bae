import { useEffect, useState } from 'react'
import { Mascot } from '#/components/Mascot'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import type { Income } from '#/data/api'
import { moneyToNumber } from '#/data/api'
import { formatCurrency, formatDateMMDDYYYY } from '#/data/format'
import { useMultiSelect } from '#/hooks/useMultiSelect'

interface ImportIncomeDialogProps {
  open: boolean
  candidates: Income[]
  onClose: () => void
  onConfirm: (ids: string[]) => Promise<void>
}

// Lets the ledger detail page attach existing unassigned income (rather
// than creating new income records) — see _app.ledger.$periodId.tsx,
// which supplies `candidates` already filtered to this ledger's month.
export function ImportIncomeDialog({
  open,
  candidates,
  onClose,
  onConfirm,
}: ImportIncomeDialogProps) {
  const selection = useMultiSelect()
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (open) selection.clear()
  }, [open])

  if (!open) return null

  async function handleConfirm() {
    setSaving(true)
    try {
      await onConfirm(selection.selectedIds)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center p-4"
      style={{ background: 'rgba(16,17,32,.62)' }}
    >
      <div
        className="dialog elev-lg relative"
        style={{ background: '#1b1d2e', width: 430 }}
      >
        <div
          className="absolute -top-[30px] right-[14px]"
          style={{ width: 56 }}
        >
          <Mascot size={56} />
        </div>
        <div className="dialog-title mono" style={{ fontSize: 17 }}>
          Import income
        </div>

        {candidates.length === 0 ? (
          <div
            className="mono"
            style={{
              fontSize: 13,
              color: 'rgba(233,233,237,.55)',
              padding: '8px 0',
            }}
          >
            No unassigned income for this month.
          </div>
        ) : (
          <table className="table">
            <thead>
              <tr>
                <th style={{ width: 36 }} />
                <th>Date</th>
                <th style={{ textAlign: 'right' }}>Amount</th>
              </tr>
            </thead>
            <tbody>
              {candidates.map((income) => (
                <tr
                  key={income.id}
                  className="mb-row"
                  onClick={() => selection.toggle(income.id)}
                  style={{
                    background: selection.isSelected(income.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                >
                  <td>
                    <SelectCheckbox
                      checked={selection.isSelected(income.id)}
                      onToggle={() => selection.toggle(income.id)}
                    />
                  </td>
                  <td className="mono">{formatDateMMDDYYYY(income.date)}</td>
                  <td className="mono" style={{ textAlign: 'right' }}>
                    {formatCurrency(moneyToNumber(income.amount))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        <div className="dialog-actions">
          <button
            className="btn btn-secondary mono"
            onClick={onClose}
            style={{ fontSize: 12.5 }}
          >
            Cancel
          </button>
          <button
            className="btn btn-primary mono"
            onClick={() => void handleConfirm()}
            disabled={selection.count === 0 || saving}
            style={{ fontSize: 12.5 }}
          >
            Attach{selection.count > 0 ? ` (${selection.count})` : ''}
          </button>
        </div>
      </div>
    </div>
  )
}
