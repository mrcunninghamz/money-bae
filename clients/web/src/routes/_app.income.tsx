import { useEffect } from 'react'
import {
  createColumnHelper,
  createPaginatedRowModel,
  rowPaginationFeature,
  tableFeatures,
  useTable,
} from '@tanstack/react-table'
import { createFileRoute } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import type { Income } from '#/data/api'
import { moneyToNumber } from '#/data/api'
import { formatCurrency, formatDateMMDDYYYY } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/income')({
  component: IncomePage,
})

const features = tableFeatures({
  rowPaginationFeature,
  paginatedRowModel: createPaginatedRowModel(),
})

const columnHelper = createColumnHelper<typeof features, Income>()

const HEADER_CELL_STYLE = [
  { paddingLeft: 16, width: 36 },
  {},
  { textAlign: 'right' as const },
  { paddingRight: 16 },
]

const BODY_CELL_STYLE = [
  { paddingLeft: 16 },
  {},
  { textAlign: 'right' as const },
  { paddingRight: 16, color: 'rgba(233,233,237,.45)', fontSize: 12.5 },
]

const MONO_COLUMN_INDEXES = new Set([1, 2])

function IncomePage() {
  const store = useAppStore()
  const selection = useMultiSelect()

  useEffect(() => {
    void store.ensureIncome()
  }, [])

  const columns = columnHelper.columns([
    columnHelper.display({
      id: 'select',
      header: () => null,
      cell: (info) => (
        <SelectCheckbox
          checked={selection.isSelected(info.row.original.id)}
          onToggle={() => selection.toggle(info.row.original.id)}
        />
      ),
    }),
    columnHelper.accessor('date', {
      header: 'Date',
      cell: (info) => formatDateMMDDYYYY(info.getValue()),
    }),
    columnHelper.accessor('amount', {
      header: 'Amount',
      cell: (info) => formatCurrency(moneyToNumber(info.getValue())),
    }),
    columnHelper.accessor('notes', {
      header: 'Notes',
      cell: (info) => info.getValue() ?? '',
    }),
  ])

  const table = useTable({
    features,
    columns,
    data: store.income,
    initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
  })

  function openEdit(id: string) {
    store.selectIncome(id)
    store.openIncomeModal('Edit')
  }

  function handleEdit() {
    const [id] = selection.selectedIds
    if (!id) return
    openEdit(id)
  }

  function handleDuplicate() {
    const [id] = selection.selectedIds
    if (!id) return
    void store.duplicateIncomeEntry(id)
    selection.clear()
  }

  function handleDelete() {
    const ids = selection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteIncomeEntries(ids)
      selection.clear()
    })
  }

  return (
    <>
      <PageHeader
        kicker="deposits"
        title="Income"
        actions={
          <EntityActionBar
            selectedCount={selection.count}
            onAdd={() => store.openIncomeModal('Add')}
            onEdit={handleEdit}
            onDuplicate={handleDuplicate}
            onDelete={handleDelete}
          />
        }
      />
      {/*
        The "weekly cadence" sidebar (typical check, next deposit, 3-paycheck
        cycle flag) was static mockup copy with no real data behind it —
        removed until an income summary agent can generate it from actual
        history. See issue #51. Restore the two-column grid
        (gridTemplateColumns: '1.6fr 1fr') and the sidebar card alongside
        this table once that exists.
      */}
      <div
        className="grid min-w-0 flex-1 items-start gap-[18px]"
        style={{ padding: '20px 24px 8px', gridTemplateColumns: '1fr' }}
      >
        <div
          className="card elev-sm overflow-hidden"
          style={{ background: '#1b1d2e', padding: 0, gap: 0 }}
        >
          <table className="table">
            <thead>
              {table.getHeaderGroups().map((headerGroup) => (
                <tr key={headerGroup.id}>
                  {headerGroup.headers.map((header, i) => (
                    <th key={header.id} style={HEADER_CELL_STYLE[i]}>
                      {header.isPlaceholder ? null : (
                        <table.FlexRender header={header} />
                      )}
                    </th>
                  ))}
                </tr>
              ))}
            </thead>
            <tbody>
              {table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  className="mb-row"
                  onClick={() => openEdit(row.original.id)}
                >
                  {row.getAllCells().map((cell, i) => (
                    <td
                      key={cell.id}
                      className={
                        MONO_COLUMN_INDEXES.has(i) ? 'mono' : undefined
                      }
                      style={BODY_CELL_STYLE[i]}
                    >
                      <table.FlexRender cell={cell} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
          <div
            className="flex items-center justify-end gap-[8px]"
            style={{
              padding: '12px 16px',
            }}
          >
            <span
              className="mono"
              style={{ fontSize: 12, color: 'rgba(233,233,237,.5)' }}
            >
              Page {table.state.pagination.pageIndex + 1} of{' '}
              {table.getPageCount() || 1}
            </span>
            <button
              className="btn btn-secondary mono"
              style={{ fontSize: 12 }}
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              Prev
            </button>
            <button
              className="btn btn-secondary mono"
              style={{ fontSize: 12 }}
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              Next
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
