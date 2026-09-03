package auth

import (
	"context"

	"github.com/coreos/go-oidc/v3/oidc"
	"gorm.io/gorm"
)

var _ Verifier = (*OIDCVerifier)(nil)

type OIDCVerifier struct {
	verifier *oidc.IDTokenVerifier
	db       *gorm.DB
}

func NewOIDCVerifier(ctx context.Context, issuerURL, audience string, db *gorm.DB) (*OIDCVerifier, error) {
	provider, err := oidc.NewProvider(ctx, issuerURL)
	if err != nil {
		return nil, err
	}
	return &OIDCVerifier{
		verifier: provider.Verifier(&oidc.Config{ClientID: audience}),
		db:       db,
	}, nil
}

func (v *OIDCVerifier) Verify(ctx context.Context, rawIDToken string) (*UserPrincipal, error) {
	idToken, err := v.verifier.Verify(ctx, rawIDToken)
	if err != nil {
		return nil, err
	}

	var claims Claims
	if err := idToken.Claims(&claims); err != nil {
		return nil, err
	}

	userID, err := resolveUserID(v.db, claims)
	if err != nil {
		return nil, err
	}

	return &UserPrincipal{UserID: userID, Sub: claims.Sub, Email: claims.Email}, nil
}
