package migrations

import (
	"github.com/go-gormigrate/gormigrate/v2"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

// All holds every versioned migration added after the initial schema below.
// New schema changes get appended here, each with its own ID/Migrate/Rollback
// — never edit an already-applied entry.
var All = []*gormigrate.Migration{}

// Run applies the initial schema (via InitSchema, GORM's AutoMigrate against
// the current models — safe even if some of these tables already exist from
// an earlier plain AutoMigrate call, since AutoMigrate only adds what's
// missing) plus any migrations in All, tracked in a "migrations" table so
// this is safe to call on every startup.
func Run(db *gorm.DB) error {
	m := gormigrate.New(db, gormigrate.DefaultOptions, All)
	m.InitSchema(func(tx *gorm.DB) error {
		return tx.AutoMigrate(
			&models.User{},
			&models.Bill{},
			&models.Income{},
			&models.Ledger{},
			&models.Pto{},
			&models.LedgerBill{},
			&models.PtoPlan{},
			&models.HolidayHour{},
		)
	})
	return m.Migrate()
}
