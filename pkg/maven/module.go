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
	"github.com/enseadaio/enseada/pkg/app"
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

func NewModule(logger log.Logger, e *echo.Echo, data *kivik.Client, storage storage.Backend, enf *casbin.Enforcer, s *auth.Store, op fosite.OAuth2Provider) (*Module, error) {
	mvn := maven.New(logger, data, storage)
	mountRoutes(e, mvn, s, op, enf)
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

func (m *Module) EventHandlers() app.EventHandlersMap {
	return app.EventHandlersMap{
		app.BeforeApplicationStartEvent: m.beforeAppStart,
	}
}

func (m *Module) beforeAppStart(ctx context.Context, event app.LifecycleEvent) error {
	if err := couch.Transact(ctx, m.Maven.Logger, m.data, migrateDB, couch.MavenDB); err != nil {
		return err
	}

	return nil
}
