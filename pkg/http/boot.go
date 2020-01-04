// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"context"

	enseada "github.com/enseadaio/enseada/pkg"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/labstack/echo"
	goauth "golang.org/x/oauth2"
)

func Boot(_ context.Context, logger log.Logger, oc *goauth.Config, skb []byte) (*echo.Echo, enseada.StopFunc, error) {
	e := createEchoServer(logger)

	mountHealthCheck(e)
	mountUI(e, oc, middleware.Session(skb))
	return e, func(ctx context.Context) error {
		e.Logger.Info("Shutting down server...")
		return e.Shutdown(ctx)
	}, nil
}
