// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"github.com/casbin/casbin/v2"
	"github.com/chartmuseum/storage"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Maven struct {
	logger   echo.Logger
	data     *kivik.Client
	storage  storage.Backend
	enforcer *casbin.Enforcer
}

func New(logger echo.Logger, data *kivik.Client, storage storage.Backend, enforcer *casbin.Enforcer) *Maven {
	return &Maven{
		logger:   logger,
		data:     data,
		storage:  storage,
		enforcer: enforcer,
	}
}
