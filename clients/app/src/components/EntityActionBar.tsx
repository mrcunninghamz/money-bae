interface EntityActionBarProps {
  selectedCount: number
  addLabel?: string
  onAdd?: () => void
  onEdit?: () => void
  onDuplicate?: () => void
  onDelete?: () => void
}

// Shared action buttons for every selectable list in the app (income,
// bills, PTO, holidays, ledger sub-tables): Add, Edit, Duplicate, Delete —
// in that order, Delete always last. Edit/Duplicate only make sense for
// exactly one selected row; Delete works for one or more. Renders just the
// buttons (no wrapper) so callers can place them in a PageHeader's actions
// slot or a card's own header row — kept near the top of the page/card so
// they stay visible above a long table.
export function EntityActionBar({
  selectedCount,
  addLabel = '+ Add',
  onAdd,
  onEdit,
  onDuplicate,
  onDelete,
}: EntityActionBarProps) {
  return (
    <>
      {onAdd && (
        <button
          className="btn btn-primary mono"
          style={{ fontSize: 12.5 }}
          onClick={onAdd}
        >
          {addLabel}
        </button>
      )}
      {onEdit && (
        <button
          className="btn btn-secondary mono"
          style={{ fontSize: 12.5 }}
          disabled={selectedCount !== 1}
          onClick={onEdit}
        >
          Edit
        </button>
      )}
      {onDuplicate && (
        <button
          className="btn btn-secondary mono"
          style={{ fontSize: 12.5 }}
          disabled={selectedCount !== 1}
          onClick={onDuplicate}
        >
          Duplicate
        </button>
      )}
      {onDelete && (
        <button
          className="btn btn-secondary mono"
          style={{ fontSize: 12.5 }}
          disabled={selectedCount === 0}
          onClick={onDelete}
        >
          Delete
        </button>
      )}
    </>
  )
}
