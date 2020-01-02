// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"time"

	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
	"github.com/ory/fosite/token/jwt"
)

func NewSession(u *User, audiences ...string) fosite.Session {
	if len(audiences) == 0 {
		audiences = []string{"enseada"}
	}

	if u == nil {
		return &openid.DefaultSession{
			Claims: &jwt.IDTokenClaims{
				Issuer:      "enseada",
				Subject:     "",
				Audience:    audiences,
				Nonce:       "",
				ExpiresAt:   time.Now().Add(time.Hour * 6),
				IssuedAt:    time.Now(),
				RequestedAt: time.Now(),
				AuthTime:    time.Now(),
			},
			Username: "",
			Subject:  "",
		}
	}

	return &openid.DefaultSession{
		Claims: &jwt.IDTokenClaims{
			Issuer:      "enseada",
			Subject:     u.Username,
			Audience:    audiences,
			Nonce:       "",
			ExpiresAt:   time.Now().Add(time.Hour * 6),
			IssuedAt:    time.Now(),
			RequestedAt: time.Now(),
			AuthTime:    time.Now(),
			Extra: echo.Map{
				"username": u.Username,
			},
		},
		Username: u.Username,
		Subject:  u.Username,
	}
}
