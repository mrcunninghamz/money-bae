package auth

import (
	"errors"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

// resolveUserID finds the user with the given claims' Sub, creating one if
// it doesn't exist yet, and returns its ID. Shared by every Verifier
// implementation so "find-or-create by sub" has exactly one definition.
func resolveUserID(db *gorm.DB, claims Claims) (uuid.UUID, error) {
	var user models.User
	err := db.Where("sub = ?", claims.Sub).First(&user).Error
	switch {
	case errors.Is(err, gorm.ErrRecordNotFound):
		user = models.User{Sub: claims.Sub, Email: claims.Email}
		if err := db.Create(&user).Error; err != nil {
			return uuid.Nil, err
		}
		return user.ID, nil
	case err != nil:
		return uuid.Nil, err
	default:
		return user.ID, nil
	}
}
