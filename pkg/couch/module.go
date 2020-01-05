// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package couch

import (
	"context"
	"errors"

	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/couchdb"
	"github.com/go-kivik/kivik"
)

type Module struct {
	logger log.Logger
	Data   *kivik.Client
}

func NewModule(ctx context.Context, logger log.Logger, url string, user string, pwd string) (*Module, error) {
	data, err := kivik.New("couch", url)
	if err != nil {
		return nil, err
	}

	up, err := data.Ping(ctx)
	if err != nil {
		return nil, err
	}

	if !up {
		return nil, errors.New("database not available")
	}

	if err := data.Authenticate(ctx, couchdb.BasicAuth(user, pwd)); err != nil {
		return nil, err
	}
	logger.Debug("database authentication successful")

	return &Module{
		logger: logger,
		Data:   data,
	}, nil
}

func (m *Module) Start(ctx context.Context) error {
	m.logger.Info("started couch module")
	return nil
}

func (m *Module) Stop(ctx context.Context) error {
	if err := m.Data.Close(ctx); err != nil {
		return err
	}

	m.logger.Info("stopped couch module")
	return nil
}
