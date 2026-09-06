package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type Ledger struct {
	Base
	UserID      uuid.UUID       `gorm:"not null;index"`
	User        User            `gorm:"foreignKey:UserID"`
	Date        time.Time       `gorm:"not null"`
	BankBalance decimal.Decimal `gorm:"not null;type:numeric"`
	Income      decimal.Decimal `gorm:"not null;type:numeric"`
	Expenses    decimal.Decimal `gorm:"not null;type:numeric"`
	Net         *decimal.Decimal `gorm:"type:numeric"`
	Name        *string
	Total       *decimal.Decimal `gorm:"type:numeric"`
	Notes       *string
	Incomes     []Income     `gorm:"foreignKey:LedgerID"`
	LedgerBills []LedgerBill `gorm:"foreignKey:LedgerID"`
}
