import { useEffect } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import { moneyToNumber } from '#/data/api'
import { formatCurrency } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/bills')({
  component: BillsPage,
})

function BillsPage() {
  const store = useAppStore()
  const selection = useMultiSelect()

  useEffect(() => {
    void store.ensureBills()
  }, [])

  function handleEdit() {
    const [id] = selection.selectedIds
    if (id) store.selectBill(id)
    if (id) store.openBillModal('Edit')
  }

  function handleDuplicate() {
    const [id] = selection.selectedIds
    if (!id) return
    void store.duplicateBillEntry(id)
    selection.clear()
  }

  function handleDelete() {
    const ids = selection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteBillEntries(ids)
      selection.clear()
    })
  }

  return (
    <>
      <PageHeader
        kicker="recurring"
        title="Bills"
        actions={
          <EntityActionBar
            selectedCount={selection.count}
            onAdd={() => store.openBillModal('Add')}
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
                <th>Name</th>
                <th style={{ textAlign: 'right' }}>Amount</th>
                <th>Due day</th>
                <th>Auto pay</th>
                <th style={{ paddingRight: 16 }}>Notes</th>
              </tr>
            </thead>
            <tbody>
              {store.bills.map((bill) => (
                <tr
                  key={bill.id}
                  className="mb-row"
                  onClick={() => {
                    store.selectBill(bill.id)
                    store.openBillModal('Edit')
                  }}
                  style={{
                    background: selection.isSelected(bill.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                >
                  <td style={{ paddingLeft: 16 }}>
                    <SelectCheckbox
                      checked={selection.isSelected(bill.id)}
                      onToggle={() => selection.toggle(bill.id)}
                    />
                  </td>
                  <td>{bill.name}</td>
                  <td className="mono" style={{ textAlign: 'right' }}>
                    {formatCurrency(moneyToNumber(bill.amount))}
                  </td>
                  <td
                    className="mono"
                    style={{ color: 'rgba(233,233,237,.6)' }}
                  >
                    {bill.dueDay ?? '—'}
                  </td>
                  <td>
                    <span
                      className={`tag mono ${bill.isAutoPay ? 'tag-accent' : 'tag-neutral'}`}
                    >
                      {bill.isAutoPay ? 'auto' : 'manual'}
                    </span>
                  </td>
                  <td
                    style={{
                      paddingRight: 16,
                      color: 'rgba(233,233,237,.45)',
                      fontSize: 12.5,
                    }}
                  >
                    {bill.notes}
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
