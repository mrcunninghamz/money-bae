package auth

import (
	"context"

	"gorm.io/gorm"
)

// SeedUserSub and SeedUserEmail identify the single seeded user created by
// cmd/import-legacy-data — the only user in the system until a real IdP is
// wired in.
const (
	SeedUserSub   = "mlEnvoOl8QCY6CZ3ul0MD7i7WLDwbCRUW2PvUuM4bqU"
	SeedUserEmail = "kmerecido@gmail.com"
)

var _ Verifier = (*MockVerifier)(nil)

// MockVerifier ignores the incoming token and always authenticates as the
// seed user. A stand-in for OIDCVerifier until a real IdP is chosen.
type MockVerifier struct {
	DB *gorm.DB
}

func (v MockVerifier) Verify(ctx context.Context, rawIDToken string) (*UserPrincipal, error) {
	claims := Claims{Sub: SeedUserSub, Email: SeedUserEmail}
	userID, err := resolveUserID(v.DB, claims)
	if err != nil {
		return nil, err
	}
	return &UserPrincipal{UserID: userID, Sub: claims.Sub, Email: claims.Email}, nil
}
