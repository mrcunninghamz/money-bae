import { useEffect } from 'react'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import { moneyToNumber } from '#/data/api'
import { formatCurrency, formatDateMMDDYYYY } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/ledger/')({
  component: LedgerListPage,
})

function LedgerListPage() {
  const store = useAppStore()
  const navigate = useNavigate()
  const selection = useMultiSelect()

  useEffect(() => {
    void store.ensureLedgers()
  }, [])

  function handleEdit() {
    const [id] = selection.selectedIds
    if (id) store.selectLedger(id)
    if (id) store.openLedgerModal('Edit')
  }

  function handleDuplicate() {
    const [id] = selection.selectedIds
    if (!id) return
    void store.duplicateLedgerEntry(id)
    selection.clear()
  }

  function handleDelete() {
    const ids = selection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteLedgerEntries(ids)
      selection.clear()
    })
  }

  return (
    <>
      <PageHeader
        kicker="all cycles"
        title="Ledger"
        actions={
          <EntityActionBar
            selectedCount={selection.count}
            onAdd={() => store.openLedgerModal('Add')}
            onEdit={handleEdit}
            onDuplicate={handleDuplicate}
            onDelete={handleDelete}
          />
        }
      />
      <div className="min-w-0 flex-1" style={{ padding: '20px 24px 8px' }}>
        <div
          className="card elev-sm overflow-hidden"
          style={{ background: '#1b1d2e', padding: 0, gap: 0 }}
        >
          <table className="table">
            <thead>
              <tr>
                <th style={{ paddingLeft: 16, width: 36 }} />
                <th>Date</th>
                <th>Name</th>
                <th style={{ textAlign: 'right' }}>Bank balance</th>
                <th style={{ textAlign: 'right' }}>Expenses</th>
                <th style={{ textAlign: 'right', paddingRight: 16 }}>Net</th>
              </tr>
            </thead>
            <tbody>
              {store.ledgers.map((row) => (
                <tr
                  key={row.id}
                  className="mb-row"
                  onClick={() =>
                    navigate({
                      to: '/ledger/$periodId',
                      params: { periodId: row.id },
                    })
                  }
                  style={{
                    background: selection.isSelected(row.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                >
                  <td style={{ paddingLeft: 16 }}>
                    <SelectCheckbox
                      checked={selection.isSelected(row.id)}
                      onToggle={() => selection.toggle(row.id)}
                    />
                  </td>
                  <td
                    className="mono"
                    style={{ color: 'rgba(233,233,237,.6)' }}
                  >
                    {formatDateMMDDYYYY(row.date)}
                  </td>
                  <td>{row.name ?? '—'}</td>
                  <td className="mono" style={{ textAlign: 'right' }}>
                    {formatCurrency(moneyToNumber(row.bankBalance))}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      color: 'rgba(233,233,237,.7)',
                    }}
                  >
                    {formatCurrency(moneyToNumber(row.expenses))}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      paddingRight: 16,
                      color: moneyToNumber(row.net) < 0 ? '#b5abfc' : '#e9e9ed',
                    }}
                  >
                    {formatCurrency(moneyToNumber(row.net))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </>
  )
}
