package auth

import (
	"context"

	"github.com/coreos/go-oidc/v3/oidc"
)

type OIDCVerifier struct {
	verifier *oidc.IDTokenVerifier
}

func NewOIDCVerifier(ctx context.Context, issuerURL, audience string) (*OIDCVerifier, error) {
	provider, err := oidc.NewProvider(ctx, issuerURL)
	if err != nil {
		return nil, err
	}
	return &OIDCVerifier{
		verifier: provider.Verifier(&oidc.Config{ClientID: audience}),
	}, nil
}

func (v *OIDCVerifier) Verify(ctx context.Context, rawIDToken string) (*Claims, error) {
	idToken, err := v.verifier.Verify(ctx, rawIDToken)
	if err != nil {
		return nil, err
	}

	var claims struct {
		Sub   string `json:"sub"`
		Email string `json:"email"`
	}
	if err := idToken.Claims(&claims); err != nil {
		return nil, err
	}

	return &Claims{Sub: claims.Sub, Email: claims.Email}, nil
}
