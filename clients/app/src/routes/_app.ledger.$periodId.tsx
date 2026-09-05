import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import type { LedgerDetail } from '#/data/api'
import { getLedger, moneyToNumber, updateLedgerBill } from '#/data/api'
import { formatCurrency, formatDateMMDDYYYY } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/ledger/$periodId')({
  component: LedgerItemPage,
})

function LedgerItemPage() {
  const { periodId } = Route.useParams()
  const store = useAppStore()
  const billSelection = useMultiSelect()
  const incomeSelection = useMultiSelect()
  const [detail, setDetail] = useState<LedgerDetail | null>(null)
  const [loadError, setLoadError] = useState(false)

  function reload() {
    getLedger(periodId)
      .then(setDetail)
      .catch((err: unknown) => {
        console.error('failed to load ledger', err)
        setLoadError(true)
      })
  }

  useEffect(() => {
    void store.ensureBills()
  }, [])

  useEffect(() => {
    store.setActiveLedger(periodId)
    setDetail(null)
    setLoadError(false)
    reload()
    return () => store.setActiveLedger(null)
  }, [periodId])

  // EditModal's LedgerBill add/edit saves directly through the API (no
  // route-local state to update) — reload once the modal closes so this
  // page's local `detail` picks up the change.
  useEffect(() => {
    if (store.modal === null) reload()
  }, [store.modal])

  async function handleTogglePaid(ledgerBillId: string) {
    if (!detail) return
    const lb = detail.ledgerBills.find((b) => b.id === ledgerBillId)
    if (!lb) return
    try {
      await updateLedgerBill(periodId, lb.id, {
        billId: lb.billId,
        amount: lb.amount,
        dueDay: lb.dueDay,
        isPayed: !lb.isPayed,
        notes: lb.notes,
      })
      reload()
    } catch (err) {
      console.error('failed to update ledger bill', err)
      store.showToast('error', "couldn't save that — try again")
    }
  }

  async function handleMarkAllPaid() {
    if (!detail) return
    try {
      await Promise.all(
        detail.ledgerBills
          .filter((lb) => !lb.isPayed)
          .map((lb) =>
            updateLedgerBill(periodId, lb.id, {
              billId: lb.billId,
              amount: lb.amount,
              dueDay: lb.dueDay,
              isPayed: true,
              notes: lb.notes,
            }),
          ),
      )
      reload()
    } catch (err) {
      console.error('failed to mark all paid', err)
      store.showToast('error', "couldn't save that — try again")
    }
  }

  function handleEditBill() {
    if (!detail) return
    const [id] = billSelection.selectedIds
    const entry = detail.ledgerBills.find((lb) => lb.id === id)
    if (entry) {
      store.selectLedgerBill(entry)
      store.openLedgerBillModal('Edit')
    }
  }

  function handleDeleteBills() {
    const ids = billSelection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteLedgerBillEntries(periodId, ids).then(reload)
      billSelection.clear()
    })
  }

  function handleDeleteIncomes() {
    const ids = incomeSelection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteIncomeEntries(ids).then(reload)
      incomeSelection.clear()
    })
  }

  if (loadError) {
    return (
      <>
        <PageHeader kicker="ledger item" title="Not found" />
        <div style={{ padding: '20px 24px' }}>
          Couldn&apos;t load this ledger cycle.
        </div>
      </>
    )
  }

  if (!detail) return null

  return (
    <>
      <PageHeader
        kicker="ledger item"
        title={`${detail.name ?? 'Ledger cycle'} · ${formatDateMMDDYYYY(detail.date)}`}
      />
      <div
        className="grid min-w-0 flex-1 items-start gap-[18px]"
        style={{ padding: '20px 24px 8px', gridTemplateColumns: '1.5fr 1fr' }}
      >
        <div
          className="card elev-sm overflow-hidden"
          style={{ background: '#1b1d2e', padding: 0, gap: 0 }}
        >
          <div
            className="flex items-center gap-[8px]"
            style={{ padding: '13px 16px' }}
          >
            <span className="card-kicker mono">bills in this cycle</span>
            <span
              className="mono"
              style={{ fontSize: 11, color: 'rgba(233,233,237,.4)' }}
            >
              click a row to toggle paid
            </span>
            <div className="flex gap-[8px] ml-auto">
              <EntityActionBar
                selectedCount={billSelection.count}
                addLabel="+ Add bill"
                onAdd={() => store.openLedgerBillModal('Add')}
                onEdit={handleEditBill}
                onDelete={handleDeleteBills}
              />
            </div>
          </div>
          <table className="table">
            <thead>
              <tr>
                <th style={{ paddingLeft: 16, width: 36 }} />
                <th>Bill</th>
                <th style={{ textAlign: 'right' }}>Amount</th>
                <th>Due</th>
                <th style={{ textAlign: 'right', paddingRight: 16 }}>Paid</th>
              </tr>
            </thead>
            <tbody>
              {detail.ledgerBills.map((lb) => (
                <tr
                  key={lb.id}
                  className="mb-row"
                  onClick={() => void handleTogglePaid(lb.id)}
                  style={{
                    background: billSelection.isSelected(lb.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                >
                  <td style={{ paddingLeft: 16 }}>
                    <SelectCheckbox
                      checked={billSelection.isSelected(lb.id)}
                      onToggle={() => billSelection.toggle(lb.id)}
                    />
                  </td>
                  <td>{lb.bill.name}</td>
                  <td className="mono" style={{ textAlign: 'right' }}>
                    {formatCurrency(moneyToNumber(lb.amount))}
                  </td>
                  <td
                    className="mono"
                    style={{ color: 'rgba(233,233,237,.55)' }}
                  >
                    {lb.dueDay ?? '—'}
                  </td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      paddingRight: 16,
                      color: lb.isPayed ? '#b5abfc' : 'rgba(233,233,237,.3)',
                    }}
                  >
                    {lb.isPayed ? '✓' : '·'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="flex gap-[8px]" style={{ padding: '12px 16px' }}>
            <button
              className="btn btn-ghost mono"
              style={{ fontSize: 12 }}
              onClick={() => void handleMarkAllPaid()}
            >
              Mark all paid
            </button>
          </div>
        </div>

        <div className="flex flex-col gap-[18px]">
          <div
            className="card elev-sm overflow-hidden"
            style={{ background: '#1b1d2e', padding: 0, gap: 0 }}
          >
            <div
              className="flex items-center gap-[8px]"
              style={{ padding: '13px 16px' }}
            >
              <span className="card-kicker mono">incomes</span>
              <div className="flex gap-[8px] ml-auto">
                <EntityActionBar
                  selectedCount={incomeSelection.count}
                  addLabel="+ Add income"
                  onAdd={() => store.openIncomeModal('Add')}
                  onDelete={handleDeleteIncomes}
                />
              </div>
            </div>
            <table className="table">
              <thead>
                <tr>
                  <th style={{ paddingLeft: 16, width: 36 }} />
                  <th>Date</th>
                  <th style={{ textAlign: 'right', paddingRight: 16 }}>
                    Amount
                  </th>
                </tr>
              </thead>
              <tbody>
                {detail.incomes.map((income) => (
                  <tr
                    key={income.id}
                    className="mb-row"
                    onClick={() => incomeSelection.toggle(income.id)}
                    style={{
                      background: incomeSelection.isSelected(income.id)
                        ? 'rgba(145,132,217,.16)'
                        : 'transparent',
                    }}
                  >
                    <td style={{ paddingLeft: 16 }}>
                      <SelectCheckbox
                        checked={incomeSelection.isSelected(income.id)}
                        onToggle={() => incomeSelection.toggle(income.id)}
                      />
                    </td>
                    <td className="mono">{formatDateMMDDYYYY(income.date)}</td>
                    <td
                      className="mono"
                      style={{ textAlign: 'right', paddingRight: 16 }}
                    >
                      {formatCurrency(moneyToNumber(income.amount))}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 11 }}
          >
            <span className="card-kicker mono">summary</span>
            <div
              className="mono flex flex-col gap-[6px]"
              style={{ fontSize: 13 }}
            >
              <div className="flex">
                <span
                  className="flex-1"
                  style={{ color: 'rgba(233,233,237,.6)' }}
                >
                  Bank balance
                </span>
                <span>{formatCurrency(moneyToNumber(detail.bankBalance))}</span>
              </div>
              <div className="flex">
                <span
                  className="flex-1"
                  style={{ color: 'rgba(233,233,237,.6)' }}
                >
                  Income ({detail.incomes.length} items)
                </span>
                <span>{formatCurrency(moneyToNumber(detail.income))}</span>
              </div>
              <div className="flex">
                <span
                  className="flex-1"
                  style={{ color: 'rgba(233,233,237,.6)' }}
                >
                  Expenses
                </span>
                <span>{formatCurrency(moneyToNumber(detail.expenses))}</span>
              </div>
            </div>
            <div
              className="h-px"
              style={{
                background:
                  'linear-gradient(to right,transparent,rgba(145,132,217,.5),transparent)',
              }}
            />
            <div className="flex items-baseline">
              <span
                className="mono flex-1"
                style={{
                  fontSize: 11,
                  letterSpacing: '.16em',
                  textTransform: 'uppercase',
                  color: '#9184d9',
                }}
              >
                net
              </span>
              <span className="mono" style={{ fontSize: 24 }}>
                {formatCurrency(moneyToNumber(detail.net))}
              </span>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
