// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package middleware

import (
	"time"

	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
)

func Session(skb []byte) echo.MiddlewareFunc {
	exp := (time.Hour * 720).Seconds()
	s := session.NewCookieStore(skb)
	s.Options(session.Options{
		MaxAge:   int(exp),
		HttpOnly: true,
	})
	return session.Sessions("enseada-session", s)
}
