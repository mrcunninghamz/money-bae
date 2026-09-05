import { Mascot } from '#/components/Mascot'
import { useAppStore } from '#/data/store'

export function ConfirmDeleteDialog() {
  const store = useAppStore()
  const pending = store.pendingDelete
  if (!pending) return null

  const { count } = pending

  return (
    <div
      className="fixed inset-0 z-50 grid place-items-center p-4"
      style={{ background: 'rgba(16,17,32,.62)' }}
    >
      <div
        className="dialog elev-lg relative"
        style={{ background: '#1b1d2e', width: 360 }}
      >
        <div
          className="absolute -top-[30px] right-[14px]"
          style={{ width: 56 }}
        >
          <Mascot size={56} />
        </div>
        <div className="dialog-title mono" style={{ fontSize: 17 }}>
          Delete {count} {count === 1 ? 'item' : 'items'}?
        </div>
        <div
          style={{
            fontSize: 13,
            color: 'rgba(233,233,237,.6)',
            lineHeight: 1.5,
          }}
        >
          This can&apos;t be undone.
        </div>
        <div className="dialog-actions">
          <button
            className="btn btn-secondary mono"
            onClick={store.cancelDelete}
            style={{ fontSize: 12.5 }}
          >
            Cancel
          </button>
          <button
            className="btn btn-primary mono"
            onClick={store.confirmDelete}
            style={{ fontSize: 12.5 }}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  )
}
