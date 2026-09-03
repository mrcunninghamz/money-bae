package auth_test

import (
	"context"
	"testing"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func TestMockVerifier_AlwaysReturnsSeedUserPrincipal(t *testing.T) {
	db := testdb.New(t, &models.User{})

	principal, err := auth.MockVerifier{DB: db}.Verify(context.Background(), "anything, including empty")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if principal.Sub != auth.SeedUserSub || principal.Email != auth.SeedUserEmail {
		t.Fatalf("expected seed user claims, got %+v", principal)
	}

	var user models.User
	if err := db.Where("sub = ?", auth.SeedUserSub).First(&user).Error; err != nil {
		t.Fatalf("expected seed user to be provisioned: %v", err)
	}
	if principal.UserID != user.ID {
		t.Fatalf("expected principal UserID %s to match provisioned user %s", principal.UserID, user.ID)
	}
}
