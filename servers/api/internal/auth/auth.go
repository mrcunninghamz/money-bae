package auth

import "context"

type Claims struct {
	Sub   string
	Email string
}

type Verifier interface {
	Verify(ctx context.Context, rawIDToken string) (*Claims, error)
}
