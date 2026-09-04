package auth

import (
	"testing"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func TestResolveUserID_NewSub_CreatesUser(t *testing.T) {
	db := testdb.New(t, &models.User{})

	id, err := resolveUserID(db, Claims{Sub: "auth0|new", Email: "new@example.com"})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	var count int64
	db.Model(&models.User{}).Where("sub = ?", "auth0|new").Count(&count)
	if count != 1 {
		t.Fatalf("expected exactly 1 user row, got %d", count)
	}

	var user models.User
	if err := db.Where("sub = ?", "auth0|new").First(&user).Error; err != nil {
		t.Fatalf("failed to look up created user: %v", err)
	}
	if id != user.ID {
		t.Fatalf("expected resolved id %s to match created user %s", id, user.ID)
	}
}

func TestResolveUserID_ExistingSub_ReusesUser(t *testing.T) {
	db := testdb.New(t, &models.User{})
	existing := models.User{Sub: "auth0|existing", Email: "existing@example.com"}
	if err := db.Create(&existing).Error; err != nil {
		t.Fatalf("failed to seed user: %v", err)
	}

	id, err := resolveUserID(db, Claims{Sub: "auth0|existing"})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if id != existing.ID {
		t.Fatalf("expected to reuse existing user %s, got %s", existing.ID, id)
	}

	var count int64
	db.Model(&models.User{}).Where("sub = ?", "auth0|existing").Count(&count)
	if count != 1 {
		t.Fatalf("expected exactly 1 user row, got %d", count)
	}
}
