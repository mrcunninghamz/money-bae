package auth

import (
	"context"

	"github.com/google/uuid"
)

// Claims are the identity claims asserted by an ID token, before resolving
// them to a local User row.
type Claims struct {
	Sub   string `json:"sub"`
	Email string `json:"email"`
}

// UserPrincipal is the authenticated identity for a request: the resolved
// local User.ID alongside the token claims that produced it. UserID never
// comes from the token itself — it's resolved (find-or-create by Sub)
// against our own database, which is why it lives here rather than on
// Claims.
type UserPrincipal struct {
	UserID uuid.UUID
	Sub    string
	Email  string
}

type Verifier interface {
	Verify(ctx context.Context, rawIDToken string) (*UserPrincipal, error)
}
