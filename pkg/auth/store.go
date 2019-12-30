// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

type Store struct {
	data   *kivik.Client
	logger echo.Logger
	*OAuthClientStore
	*OAuthRequestStore
	*OIDCSessionStore
	*PKCERequestStore
}

func NewStore(data *kivik.Client, logger echo.Logger, cs *OAuthClientStore, rs *OAuthRequestStore, os *OIDCSessionStore, ps *PKCERequestStore) *Store {
	return &Store{
		data:              data,
		logger:            logger,
		OAuthClientStore:  cs,
		OAuthRequestStore: rs,
		OIDCSessionStore:  os,
		PKCERequestStore:  ps,
	}
}

func (s *Store) Authenticate(ctx context.Context, username string, password string) error {
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

func (s *Store) Save(ctx context.Context, u *User) error {
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
