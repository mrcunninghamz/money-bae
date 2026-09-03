package models

import (
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type Pto struct {
	Base
	UserID         uuid.UUID       `gorm:"not null;index"`
	User           User            `gorm:"foreignKey:UserID"`
	Year           int             `gorm:"not null"`
	PrevYearHours  decimal.Decimal `gorm:"not null"`
	AvailableHours decimal.Decimal `gorm:"not null"`
	HoursPlanned   decimal.Decimal `gorm:"not null"`
	HoursUsed      decimal.Decimal `gorm:"not null"`
	HoursRemaining decimal.Decimal `gorm:"not null"`
	RolloverHours  bool            `gorm:"not null"`
	PtoPlans       []PtoPlan       `gorm:"foreignKey:PtoID"`
	HolidayHours   []HolidayHour   `gorm:"foreignKey:PtoID"`
}
