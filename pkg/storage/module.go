// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package storage

import (
	"context"
	"fmt"

	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/pkg/log"
)

type Config struct {
	Provider   string
	StorageDir string
}

type Module struct {
	logger  log.Logger
	Backend storage.Backend
}

func NewModule(logger log.Logger, c Config) (*Module, error) {
	var b storage.Backend
	switch c.Provider {
	//case "s3":
	//	return storage.NewAmazonS3Backend()
	case "local":
		b = storage.NewLocalFilesystemBackend(c.StorageDir)
	default:
		return nil, fmt.Errorf("unsupported storage provider: %s", c.Provider)
	}
	return &Module{
		logger:  logger,
		Backend: b,
	}, nil
}

func (m *Module) Start(ctx context.Context) error {
	m.logger.Info("started storage module")
	return nil
}

func (m *Module) Stop(ctx context.Context) error {
	m.logger.Info("stopped storage module")
	return nil
}
