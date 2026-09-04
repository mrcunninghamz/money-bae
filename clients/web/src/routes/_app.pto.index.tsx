import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import { formatHours } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/pto/')({
  component: PtoYearsPage,
})

function PtoYearsPage() {
  const store = useAppStore()
  const navigate = useNavigate()
  const selection = useMultiSelect()

  function handleEdit() {
    const [id] = selection.selectedIds
    if (id) store.selectPtoYear(id)
    if (id) store.openPtoYearModal('Edit')
  }

  function handleDuplicate() {
    const [id] = selection.selectedIds
    if (!id) return
    void store.duplicatePtoYearEntry(id)
    selection.clear()
  }

  function handleDelete() {
    const ids = selection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deletePtoYearEntries(ids)
      selection.clear()
    })
  }

  return (
    <>
      <PageHeader
        kicker="by year"
        title="PTO records"
        actions={
          <EntityActionBar
            selectedCount={selection.count}
            onAdd={() => store.openPtoYearModal('Add')}
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
                <th>Year</th>
                <th style={{ textAlign: 'right' }}>Available</th>
                <th style={{ textAlign: 'right' }}>Planned</th>
                <th style={{ textAlign: 'right' }}>Used</th>
                <th style={{ textAlign: 'right', paddingRight: 16 }}>
                  Remaining
                </th>
              </tr>
            </thead>
            <tbody>
              {store.ptos.map((pto) => (
                <tr
                  key={pto.id}
                  className="mb-row"
                  style={{
                    background: selection.isSelected(pto.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                  onClick={() =>
                    navigate({
                      to: '/pto/$year',
                      params: { year: String(pto.year) },
                    })
                  }
                >
                  <td style={{ paddingLeft: 16 }}>
                    <SelectCheckbox
                      checked={selection.isSelected(pto.id)}
                      onToggle={() => selection.toggle(pto.id)}
                    />
                  </td>
                  <td className="mono">{pto.year}</td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      color: 'rgba(233,233,237,.7)',
                    }}
                  >
                    {formatHours(Number(pto.availableHours))}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      color: 'rgba(233,233,237,.7)',
                    }}
                  >
                    {formatHours(Number(pto.hoursPlanned))}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      color: 'rgba(233,233,237,.7)',
                    }}
                  >
                    {formatHours(Number(pto.hoursUsed))}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      paddingRight: 16,
                      color: '#b5abfc',
                    }}
                  >
                    {formatHours(Number(pto.hoursRemaining))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div
            className="flex flex-wrap items-center gap-[8px]"
            style={{ padding: '12px 16px' }}
          >
            <span
              className="mono"
              style={{ fontSize: 11, color: 'rgba(233,233,237,.4)' }}
            >
              click a year to open its record
            </span>
          </div>
        </div>
      </div>
    </>
  )
}
