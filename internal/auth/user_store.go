// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"errors"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
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
	u, err := s.GetUser(ctx, username)
	if err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return fosite.ErrNotFound
		}
		return err
	}
	return bcrypt.CompareHashAndPassword(u.HashedPassword, []byte(password))
}

func (s *UserStore) SaveUser(ctx context.Context, u *User) error {
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

	u.Username = id
	u.Rev = rev
	u.Password = ""
	return nil
}

func (s *UserStore) ListUsers(ctx context.Context) ([]*User, error) {
	db := s.data.DB(ctx, couch.UsersDB)
	rows, err := db.AllDocs(ctx, kivik.Options{
		"include_docs": true,
	})
	if err != nil {
		return nil, err
	}

	var users []*User
	for rows.Next() {
		user := new(User)
		if err := rows.ScanDoc(user); err != nil {
			return nil, err
		}

		users = append(users, user)
	}

	return users, nil
}

func (s *UserStore) GetUser(ctx context.Context, username string) (*User, error) {
	db := s.data.DB(ctx, couch.UsersDB)
	row := db.Get(ctx, username)
	var user User
	if err := row.ScanDoc(&user); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}

		return nil, err
	}

	if row.Err != nil {
		return nil, row.Err
	}

	return &user, nil
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
