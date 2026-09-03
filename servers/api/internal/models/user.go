package models

type User struct {
	Base
	Sub   string `gorm:"uniqueIndex;not null"`
	Email string `gorm:"index"`
}
