package models_test

import (
	"testing"

	"github.com/glebarez/sqlite"
	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

func setupTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open("file::memory:?cache=shared"), &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	if err := db.AutoMigrate(&models.User{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func TestUser_BeforeCreate_AssignsUUIDv7WhenZero(t *testing.T) {
	db := setupTestDB(t)

	user := &models.User{Sub: "auth0|123"}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	if user.ID == uuid.Nil {
		t.Fatal("expected ID to be assigned, got zero UUID")
	}
	if user.ID.Version() != 7 {
		t.Fatalf("expected UUID version 7, got %d", user.ID.Version())
	}
}

func TestUser_BeforeCreate_PreservesProvidedID(t *testing.T) {
	db := setupTestDB(t)

	fixedID := uuid.New()
	user := &models.User{Base: models.Base{ID: fixedID}, Sub: "auth0|456"}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	if user.ID != fixedID {
		t.Fatalf("expected ID to remain %s, got %s", fixedID, user.ID)
	}
}

func TestUser_Sub_MustBeUnique(t *testing.T) {
	db := setupTestDB(t)

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err != nil {
		t.Fatalf("failed to create first user: %v", err)
	}

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err == nil {
		t.Fatal("expected duplicate Sub to fail, got nil error")
	}
}
