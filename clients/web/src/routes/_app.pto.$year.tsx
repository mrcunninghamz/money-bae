import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { EntityActionBar } from '#/components/EntityActionBar'
import { PageHeader } from '#/components/PageHeader'
import { SelectCheckbox } from '#/components/SelectCheckbox'
import type { PtoDetail } from '#/data/api'
import { getPto } from '#/data/api'
import { formatHours } from '#/data/format'
import { useAppStore } from '#/data/store'
import { useMultiSelect } from '#/hooks/useMultiSelect'

export const Route = createFileRoute('/_app/pto/$year')({
  component: PtoYearPage,
})

function PtoYearPage() {
  const { year } = Route.useParams()
  const store = useAppStore()
  const ptoSelection = useMultiSelect()
  const holidaySelection = useMultiSelect()
  const [detail, setDetail] = useState<PtoDetail | null>(null)
  const [loadError, setLoadError] = useState(false)

  const ptoYear = store.ptos.find((p) => String(p.year) === year)

  function reload(id: string) {
    getPto(id)
      .then(setDetail)
      .catch((err: unknown) => {
        console.error('failed to load pto', err)
        setLoadError(true)
      })
  }

  useEffect(() => {
    if (!ptoYear) return
    store.setActivePto(ptoYear.id)
    setDetail(null)
    setLoadError(false)
    reload(ptoYear.id)
    return () => store.setActivePto(null)
  }, [ptoYear?.id])

  // EditModal's PtoPlan/HolidayHour add/edit save directly through the API
  // (no route-local state to update) — reload once the modal closes so
  // this page's local `detail` picks up the change.
  useEffect(() => {
    if (store.modal === null && ptoYear) reload(ptoYear.id)
  }, [store.modal])

  function handleEditPto() {
    if (!detail) return
    const [id] = ptoSelection.selectedIds
    const entry = detail.ptoPlans.find((p) => p.id === id)
    if (entry) {
      store.selectPtoPlan(entry)
      store.openPtoPlanModal('Edit')
    }
  }

  function handleDuplicatePto() {
    if (!detail) return
    const [id] = ptoSelection.selectedIds
    const original = detail.ptoPlans.find((p) => p.id === id)
    if (!original || !ptoYear) return
    void store
      .duplicatePtoPlanEntry({
        startDate: original.startDate,
        endDate: original.endDate,
        name: `${original.name} (copy)`,
        description: original.description,
        hours: original.hours,
        status: original.status,
        customHours: original.customHours,
      })
      .then((created) => {
        if (created) reload(ptoYear.id)
      })
    ptoSelection.clear()
  }

  function handleDeletePto() {
    if (!ptoYear) return
    const ids = ptoSelection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deletePtoPlanEntries(ptoYear.id, ids).then(() => {
        reload(ptoYear.id)
      })
      ptoSelection.clear()
    })
  }

  function handleEditHoliday() {
    if (!detail) return
    const [id] = holidaySelection.selectedIds
    const entry = detail.holidayHours.find((h) => h.id === id)
    if (entry) {
      store.selectHoliday(entry)
      store.openHolidayModal('Edit')
    }
  }

  function handleDuplicateHoliday() {
    if (!detail) return
    const [id] = holidaySelection.selectedIds
    const original = detail.holidayHours.find((h) => h.id === id)
    if (!original || !ptoYear) return
    void store
      .duplicateHolidayEntry({
        date: original.date,
        name: `${original.name} (copy)`,
        hours: original.hours,
      })
      .then((created) => {
        if (created) reload(ptoYear.id)
      })
    holidaySelection.clear()
  }

  function handleDeleteHoliday() {
    if (!ptoYear) return
    const ids = holidaySelection.selectedIds
    store.requestDelete(ids.length, () => {
      void store.deleteHolidayEntries(ptoYear.id, ids).then(() => {
        reload(ptoYear.id)
      })
      holidaySelection.clear()
    })
  }

  if (!ptoYear || loadError) {
    return (
      <>
        <PageHeader kicker="pto record" title="Not found" />
        <div style={{ padding: '20px 24px' }}>
          Couldn&apos;t load this PTO year.
        </div>
      </>
    )
  }

  if (!detail) return null

  const usedHours = Number(ptoYear.hoursUsed)
  const holidayHoursTotal = detail.holidayHours.reduce(
    (sum, h) => sum + Number(h.hours),
    0,
  )
  const remaining = Number(ptoYear.hoursRemaining)
  const available =
    Number(ptoYear.availableHours) + Number(ptoYear.prevYearHours)
  const barWidth = `${Math.min(100, Math.round((usedHours / (available || 1)) * 100))}%`

  return (
    <>
      <PageHeader kicker="pto record" title={`PTO — ${year}`} />
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
            <span className="card-kicker mono">planned pto — {year}</span>
            <div className="flex gap-[8px] ml-auto">
              <EntityActionBar
                selectedCount={ptoSelection.count}
                onAdd={() => store.openPtoPlanModal('Add')}
                onEdit={handleEditPto}
                onDuplicate={handleDuplicatePto}
                onDelete={handleDeletePto}
              />
            </div>
          </div>
          <table className="table">
            <thead>
              <tr>
                <th style={{ paddingLeft: 16, width: 36 }} />
                <th>Start</th>
                <th>End</th>
                <th>Name</th>
                <th style={{ textAlign: 'right' }}>Hours</th>
                <th style={{ paddingRight: 16 }}>Status</th>
              </tr>
            </thead>
            <tbody>
              {detail.ptoPlans.map((entry) => (
                <tr
                  key={entry.id}
                  className="mb-row"
                  onClick={() => {
                    store.selectPtoPlan(entry)
                    store.openPtoPlanModal('Edit')
                  }}
                  style={{
                    background: ptoSelection.isSelected(entry.id)
                      ? 'rgba(145,132,217,.16)'
                      : 'transparent',
                  }}
                >
                  <td style={{ paddingLeft: 16 }}>
                    <SelectCheckbox
                      checked={ptoSelection.isSelected(entry.id)}
                      onToggle={() => ptoSelection.toggle(entry.id)}
                    />
                  </td>
                  <td className="mono">{entry.startDate}</td>
                  <td
                    className="mono"
                    style={{ color: 'rgba(233,233,237,.6)' }}
                  >
                    {entry.endDate}
                  </td>
                  <td>{entry.name}</td>
                  <td
                    className="mono"
                    style={{
                      textAlign: 'right',
                      color: Number(entry.hours) < 0 ? '#b5abfc' : '#e9e9ed',
                    }}
                  >
                    {formatHours(Number(entry.hours))}
                  </td>
                  <td style={{ paddingRight: 16 }}>
                    <span className="tag tag-neutral mono">{entry.status}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="flex flex-col gap-[18px]">
          <div
            className="card elev-sm"
            style={{ background: '#1b1d2e', padding: '16px 18px', gap: 11 }}
          >
            <span className="card-kicker mono">balance</span>
            <div className="flex items-end gap-[8px]">
              <span className="mono" style={{ fontSize: 36, lineHeight: 1 }}>
                {formatHours(remaining)}
              </span>
              <span
                className="mono"
                style={{
                  fontSize: 12,
                  color: 'rgba(233,233,237,.5)',
                  paddingBottom: 6,
                }}
              >
                of {formatHours(available)} hrs left
              </span>
            </div>
            <div
              className="flex overflow-hidden rounded-[5px]"
              style={{ height: 10, background: '#292b31' }}
            >
              <div style={{ background: '#5d5294', width: barWidth }} />
            </div>
            <div
              className="mono flex flex-col gap-[5px]"
              style={{ fontSize: 12 }}
            >
              <div className="flex">
                <span
                  className="flex-1"
                  style={{ color: 'rgba(233,233,237,.6)' }}
                >
                  Booked
                </span>
                <span>{formatHours(usedHours)}</span>
              </div>
              <div className="flex">
                <span
                  className="flex-1"
                  style={{ color: 'rgba(233,233,237,.6)' }}
                >
                  Holiday hours
                </span>
                <span>{formatHours(holidayHoursTotal)}</span>
              </div>
            </div>
          </div>

          <div
            className="card elev-sm overflow-hidden"
            style={{ background: '#1b1d2e', padding: 0, gap: 0 }}
          >
            <div
              className="flex items-center gap-[8px]"
              style={{ padding: '13px 16px' }}
            >
              <span className="card-kicker mono">holiday hours</span>
              <div className="flex items-center gap-[8px] ml-auto">
                <EntityActionBar
                  selectedCount={holidaySelection.count}
                  onAdd={() => store.openHolidayModal('Add')}
                  onEdit={handleEditHoliday}
                  onDuplicate={handleDuplicateHoliday}
                  onDelete={handleDeleteHoliday}
                />
              </div>
            </div>
            <table className="table" style={{ fontSize: 13 }}>
              <tbody>
                {detail.holidayHours.map((holiday) => (
                  <tr
                    key={holiday.id}
                    className="mb-row"
                    onClick={() => {
                      store.selectHoliday(holiday)
                      store.openHolidayModal('Edit')
                    }}
                    style={{
                      background: holidaySelection.isSelected(holiday.id)
                        ? 'rgba(145,132,217,.16)'
                        : 'transparent',
                    }}
                  >
                    <td style={{ paddingLeft: 16, width: 36 }}>
                      <SelectCheckbox
                        checked={holidaySelection.isSelected(holiday.id)}
                        onToggle={() => holidaySelection.toggle(holiday.id)}
                      />
                    </td>
                    <td
                      className="mono"
                      style={{ color: 'rgba(233,233,237,.6)' }}
                    >
                      {holiday.date}
                    </td>
                    <td>{holiday.name}</td>
                    <td
                      className="mono"
                      style={{ textAlign: 'right', paddingRight: 16 }}
                    >
                      {formatHours(Number(holiday.hours))}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </>
  )
}
