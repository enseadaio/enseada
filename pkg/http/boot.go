// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"context"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	goauth "golang.org/x/oauth2"
)

func Boot(_ context.Context, lvl log.Lvl, oc *goauth.Config, skb []byte) (*echo.Echo, error) {
	e := createEchoServer(lvl)

	mountHealthCheck(e)
	mountUI(e, oc, middleware.Session(skb))
	return e, nil
}
