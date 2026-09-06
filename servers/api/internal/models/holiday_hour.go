package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type HolidayHour struct {
	Base
	PtoID uuid.UUID       `gorm:"not null;index"`
	Pto   Pto             `gorm:"foreignKey:PtoID"`
	Date  time.Time       `gorm:"not null"`
	Name  string          `gorm:"not null"`
	Hours decimal.Decimal `gorm:"not null;type:numeric"`
}
