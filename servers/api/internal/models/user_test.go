package models_test

import (
	"testing"

	"github.com/google/uuid"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func TestUser_BeforeCreate_AssignsUUIDv7WhenZero(t *testing.T) {
	db := testdb.New(t, &models.User{})

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
	db := testdb.New(t, &models.User{})

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
	db := testdb.New(t, &models.User{})

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err != nil {
		t.Fatalf("failed to create first user: %v", err)
	}

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err == nil {
		t.Fatal("expected duplicate Sub to fail, got nil error")
	}
}
