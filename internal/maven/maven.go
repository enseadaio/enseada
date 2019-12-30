// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"github.com/chartmuseum/storage"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Maven struct {
	logger  echo.Logger
	client  *kivik.Client
	storage storage.Backend
	dbname  string
}

const dbname = "maven2"

func New(client *kivik.Client, storage storage.Backend, logger echo.Logger) (*Maven, error) {
	return &Maven{
		logger:  logger,
		client:  client,
		storage: storage,
		dbname:  dbname,
	}, nil
}
