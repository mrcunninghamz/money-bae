package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type Income struct {
	Base
	UserID   uuid.UUID       `gorm:"not null;index"`
	User     User            `gorm:"foreignKey:UserID"`
	Date     time.Time       `gorm:"not null"`
	Amount   decimal.Decimal `gorm:"not null;type:numeric"`
	LedgerID *uuid.UUID      `gorm:"index"`
	Ledger   *Ledger         `gorm:"foreignKey:LedgerID"`
	Notes    *string
}
