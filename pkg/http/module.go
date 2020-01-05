// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"context"
	"fmt"

	"github.com/enseadaio/enseada/pkg/errare"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/labstack/echo"
	goauth "golang.org/x/oauth2"
)

type TLSConfig struct {
	KeyFile  string
	CertFile string
}

type Module struct {
	logger log.Logger
	Echo   *echo.Echo
	tls    *TLSConfig
	port   int
}

func NewModule(_ context.Context, logger log.Logger, errh errare.Handler, oc *goauth.Config, skb []byte, port int, tls *TLSConfig) (*Module, error) {
	e := createEchoServer(logger, errh)

	mountHealthCheck(e)
	mountUI(e, oc, middleware.Session(skb))
	return &Module{
		logger: logger,
		Echo:   e,
		tls:    tls,
		port:   port,
	}, nil
}

func (m Module) Start(ctx context.Context) error {
	addr := fmt.Sprintf(":%d", m.port)
	if m.tls == nil {
		m.logger.Info("started http module")
		return m.Echo.Start(addr)
	}

	m.logger.Info("started http module with TLS")
	return m.Echo.StartTLS(addr, m.tls.CertFile, m.tls.KeyFile)
}

func (m Module) Stop(ctx context.Context) error {
	m.logger.Info("stopped http module")
	return m.Echo.Shutdown(ctx)
}
