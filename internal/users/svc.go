package users

import (
	"context"
	"errors"
	"fmt"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"golang.org/x/crypto/bcrypt"
)

type UserSvc struct {
	logger echo.Logger
	data   *kivik.Client
}

func NewSvc(client *kivik.Client, logger echo.Logger) *UserSvc {
	return &UserSvc{
		data:   client,
		logger: logger,
	}
}

func (s *UserSvc) FindByUsername(ctx context.Context, username string) (*User, error) {
	return s.FindBy(ctx, couch.Query{
		"selector": couch.Query{
			"username": username,
		},
	})
}

func (s *UserSvc) FindBy(ctx context.Context, query couch.Query) (*User, error) {
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

	return nil, fmt.Errorf("user not found for query %+v", query)
}

func (s *UserSvc) Save(ctx context.Context, u *User) error {
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

func (s *UserSvc) Authenticate(ctx context.Context, username string, password string) (*User, error) {
	u, err := s.FindByUsername(ctx, username)
	if err != nil {
		return nil, err
	}

	return u, bcrypt.CompareHashAndPassword(u.HashedPassword, []byte(password))
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
