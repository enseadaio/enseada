package auth

import (
	"context"
	"errors"
	"fmt"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"golang.org/x/crypto/bcrypt"
)

type UserStore struct {
	data   *kivik.Client
	logger echo.Logger
}

func NewUserStore(data *kivik.Client, logger echo.Logger) *UserStore {
	return &UserStore{
		data:   data,
		logger: logger,
	}
}

func (s *UserStore) Authenticate(ctx context.Context, username string, password string) error {
	db := s.data.DB(ctx, couch.UsersDB)
	rows, err := db.Find(ctx, couch.Query{
		"selector": couch.Query{
			"username": username,
		},
	})
	if err != nil {
		return err
	}

	if rows.Next() {
		var u User
		if err := rows.ScanDoc(&u); err != nil {
			return err
		}

		return bcrypt.CompareHashAndPassword(u.HashedPassword, []byte(password))
	}

	return fmt.Errorf("user not found for username %s", username)
}

func (s *UserStore) Save(ctx context.Context, u *User) error {
	db := s.data.DB(ctx, couch.UsersDB)
	if u.HashedPassword == nil {
		err := hashPassword(u)
		if err != nil {
			return err
		}
	}

	id, rev, err := db.CreateDoc(ctx, u)
	if err != nil {
		return err
	}

	u.ID = id
	u.Rev = rev
	return nil
}

func (s *UserStore) FindByUsername(ctx context.Context, username string) (*User, error) {
	return s.FindBy(ctx, couch.Query{
		"selector": couch.Query{
			"username": username,
		},
	})
}

func (s *UserStore) FindBy(ctx context.Context, query couch.Query) (*User, error) {
	db := s.data.DB(ctx, couch.UsersDB)
	rows, err := db.Find(ctx, query)
	if err != nil {
		return nil, err
	}

	if rows.Next() {
		var user User
		if err := rows.ScanDoc(&user); err != nil {
			return nil, err
		}

		return &user, nil
	}

	return nil, nil
}

func hashPassword(u *User) error {
	if u.Password == "" {
		return errors.New("user password cannot be blank")
	}

	h, err := bcrypt.GenerateFromPassword([]byte(u.Password), bcrypt.DefaultCost)
	if err != nil {
		return err
	}

	u.HashedPassword = h
	return nil
}
