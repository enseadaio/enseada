// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/kivik"
)

type Maven struct {
	Logger  log.Logger
	data    *kivik.Client
	storage storage.Backend
}

func New(logger log.Logger, data *kivik.Client, storage storage.Backend) *Maven {
	return &Maven{
		Logger:  logger,
		data:    data,
		storage: storage,
	}
}
