package models

import (
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type PtoPlan struct {
	Base
	PtoID       uuid.UUID `gorm:"not null;index"`
	Pto         Pto       `gorm:"foreignKey:PtoID"`
	StartDate   time.Time `gorm:"not null"`
	EndDate     time.Time `gorm:"not null"`
	Name        string    `gorm:"not null"`
	Description *string
	Hours       decimal.Decimal `gorm:"not null;type:numeric"`
	Status      string          `gorm:"not null"`
	CustomHours bool            `gorm:"not null"`
}
