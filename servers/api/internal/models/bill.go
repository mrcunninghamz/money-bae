package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type Bill struct {
	Base
	UserID      uuid.UUID       `gorm:"not null;index"`
	User        User            `gorm:"foreignKey:UserID"`
	Name        string          `gorm:"not null"`
	Amount      decimal.Decimal `gorm:"not null;type:numeric"`
	DueDay      *time.Time
	IsAutoPay   bool `gorm:"not null"`
	Notes       *string
	LedgerBills []LedgerBill `gorm:"foreignKey:BillID"`
}
