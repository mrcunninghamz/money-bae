package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type LedgerBill struct {
	Base
	LedgerID uuid.UUID       `gorm:"not null;index"`
	Ledger   Ledger          `gorm:"foreignKey:LedgerID"`
	BillID   uuid.UUID       `gorm:"not null;index"`
	Bill     Bill            `gorm:"foreignKey:BillID"`
	Amount   decimal.Decimal `gorm:"not null"`
	DueDay   *time.Time
	IsPayed  bool `gorm:"not null"`
	Notes    *string
}
