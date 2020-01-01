// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Store struct {
	data   *kivik.Client
	logger echo.Logger
	*OAuthClientStore
	*OAuthRequestStore
	*OIDCSessionStore
	*PKCERequestStore
	*UserStore
}

func NewStore(data *kivik.Client, logger echo.Logger, cs *OAuthClientStore, rs *OAuthRequestStore, os *OIDCSessionStore, ps *PKCERequestStore, us *UserStore) *Store {
	return &Store{
		data:              data,
		logger:            logger,
		OAuthClientStore:  cs,
		OAuthRequestStore: rs,
		OIDCSessionStore:  os,
		PKCERequestStore:  ps,
		UserStore:         us,
	}
}
