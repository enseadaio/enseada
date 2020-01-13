// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"context"

	"github.com/casbin/casbin/v2"
	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
)

type Module struct {
	logger log.Logger
	Maven  *maven.Maven
	data   *kivik.Client
}

type Deps struct {
	Logger        log.Logger
	Data          *kivik.Client
	Echo          *echo.Echo
	Storage       storage.Backend
	Enforcer      *casbin.Enforcer
	AuthStore     *auth.Store
	OAuthProvider fosite.OAuth2Provider
}

func NewModule(ctx context.Context, deps Deps) (*Module, error) {
	logger := deps.Logger
	e := deps.Echo
	data := deps.Data
	st := deps.Storage
	enf := deps.Enforcer
	as := deps.AuthStore
	op := deps.OAuthProvider

	mvn := maven.New(logger, data, st)
	mountRoutes(e, mvn, as, op, enf)

	if err := couch.Transact(ctx, logger, data, migrateDB, couch.MavenDB); err != nil {
		return nil, err
	}

	return &Module{
		logger: logger,
		Maven:  mvn,
		data:   data,
	}, nil
}

func (m *Module) Start(ctx context.Context) error {
	m.logger.Info("started maven module")
	return nil
}

func (m *Module) Stop(ctx context.Context) error {
	m.logger.Info("stopped maven module")
	return nil
}
