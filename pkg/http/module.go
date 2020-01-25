// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"context"
	"fmt"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"

	"github.com/enseadaio/enseada/pkg/app"
	"github.com/enseadaio/enseada/pkg/errare"

	"github.com/enseadaio/enseada/pkg/log"
	goauth "golang.org/x/oauth2"
)

type TLSConfig struct {
	KeyFile  string
	CertFile string
}

type Module struct {
	logger log.Logger
	Server *Server
	tls    *TLSConfig
	port   int
}

type Deps struct {
	Logger          log.Logger
	Data            *kivik.Client
	ErrorHandler    errare.Handler
	OAuthClient     *goauth.Config
	SecretKeyBase   []byte
	Port            int
	TLS             *TLSConfig
	Store           *auth.Store
	OAuthProvider   fosite.OAuth2Provider
	Enforcer        *casbin.Enforcer
	MetricsRegistry auth.MetricsRegistry
	Maven           *maven.Maven
}

func NewModule(_ context.Context, deps Deps) (*Module, error) {
	logger := deps.Logger
	port := deps.Port
	tls := deps.TLS

	s := createServer(deps)
	middlewares(s)
	routes(s)

	return &Module{
		logger: logger,
		Server: s,
		tls:    tls,
		port:   port,
	}, nil
}

func (m Module) Start(ctx context.Context) error {
	addr := fmt.Sprintf(":%d", m.port)
	if m.tls != nil {
		m.logger.Info("started http module with TLS")
		return m.Server.StartTLS(addr, m.tls.CertFile, m.tls.KeyFile)
	}

	m.logger.Info("started http module")
	return m.Server.Start(addr)
}

func (m Module) Stop(ctx context.Context) error {
	m.logger.Info("stopping server")
	return m.Server.Shutdown(ctx)
}

func (m *Module) EventHandlers() app.EventHandlersMap {
	return app.EventHandlersMap{
		app.AfterApplicationStartEvent: m.afterStart,
		app.AfterApplicationStopEvent:  m.afterStop,
	}
}

func (m *Module) afterStart(_ context.Context, _ app.LifecycleEvent) {
	proto := "http"
	if m.tls != nil {
		proto = "https"
	}
	m.logger.Infof("listening on %s port %d", proto, m.port)
}

func (m *Module) afterStop(_ context.Context, _ app.LifecycleEvent) {
	m.logger.Info("stopped http module")
}
