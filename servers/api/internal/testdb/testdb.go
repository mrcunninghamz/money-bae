package testdb

import (
	"fmt"
	"testing"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// New opens a fresh, uniquely-named in-memory SQLite database for the given
// test, auto-migrating any models passed in. Each test gets its own database
// (keyed by t.Name()) so tests are isolated and the suite is safe to run
// with -count=2 or in any order.
func New(t *testing.T, models ...any) *gorm.DB {
	t.Helper()
	dsn := fmt.Sprintf("file:%s?mode=memory&cache=shared", t.Name())
	db, err := gorm.Open(sqlite.Open(dsn), &gorm.Config{Logger: logger.Discard})
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	// A SQLite "mode=memory&cache=shared" database lives as long as any
	// connection to it stays open. database/sql keeps idle connections
	// around after a test returns, so without an explicit close here the
	// same t.Name()-keyed database would still be alive (and populated) the
	// next time this test name runs in the same process — e.g. under
	// `go test -count=2`, which reuses the same t.Name() for each repeat.
	// Closing it in t.Cleanup guarantees every test run starts fresh.
	t.Cleanup(func() {
		if sqlDB, err := db.DB(); err == nil {
			_ = sqlDB.Close()
		}
	})
	if len(models) > 0 {
		if err := db.AutoMigrate(models...); err != nil {
			t.Fatalf("failed to migrate test db: %v", err)
		}
	}
	return db
}
