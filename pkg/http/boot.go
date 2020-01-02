// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"context"

	"github.com/uber-go/tally"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/labstack/echo"
	goauth "golang.org/x/oauth2"
)

func Boot(_ context.Context, logger log.Logger, stats tally.Scope, oc *goauth.Config, skb []byte) (*echo.Echo, error) {
	e := createEchoServer(logger, stats)

	mountHealthCheck(e)
	mountUI(e, oc, middleware.Session(skb))
	return e, nil
}
