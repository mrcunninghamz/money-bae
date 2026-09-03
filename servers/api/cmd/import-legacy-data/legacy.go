package main

import (
	"time"

	"github.com/shopspring/decimal"
)

// The structs below mirror tui/src/schema.rs exactly (column names/types),
// for scanning rows out of the old money_bae database. They intentionally
// don't live in internal/models — that package is the NEW schema's shape,
// not the legacy one.

type legacyBill struct {
	ID        int32
	Name      string
	Amount    decimal.Decimal
	DueDay    *time.Time
	IsAutoPay bool
	CreatedAt time.Time
	Notes     *string
}

type legacyIncome struct {
	ID        int32
	Date      time.Time
	Amount    decimal.Decimal
	CreatedAt time.Time
	LedgerID  *int32
	Notes     *string
}

type legacyLedger struct {
	ID          int32
	Date        time.Time
	BankBalance decimal.Decimal
	Income      decimal.Decimal
	Expenses    decimal.Decimal
	Net         *decimal.Decimal
	CreatedAt   time.Time
	Name        *string
	Total       *decimal.Decimal
	Notes       *string
}

type legacyLedgerBill struct {
	ID        int32
	LedgerID  int32
	BillID    int32
	Amount    decimal.Decimal
	DueDay    *time.Time
	IsPayed   bool
	CreatedAt time.Time
	Notes     *string
}

type legacyPto struct {
	ID             int32
	Year           int32
	PrevYearHours  decimal.Decimal
	AvailableHours decimal.Decimal
	HoursPlanned   decimal.Decimal
	HoursUsed      decimal.Decimal
	HoursRemaining decimal.Decimal
	RolloverHours  bool
	CreatedAt      time.Time
}

type legacyPtoPlan struct {
	ID          int32
	PtoID       int32
	StartDate   time.Time
	EndDate     time.Time
	Name        string
	Description *string
	Hours       decimal.Decimal
	Status      string
	CustomHours bool
	CreatedAt   time.Time
}

type legacyHolidayHours struct {
	ID        int32
	PtoID     int32
	Date      time.Time
	Name      string
	Hours     decimal.Decimal
	CreatedAt time.Time
}
