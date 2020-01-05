// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package couch

import (
	"context"

	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/kivik"
)

func InitDb(ctx context.Context, logger log.Logger, client *kivik.Client, name string) error {
	does, err := client.DBExists(ctx, name)
	if err != nil {
		return err
	}
	if !does {
		logger.Debugf("initializing database %s", name)
		return client.CreateDB(ctx, name)
	}
	logger.Debugf("database %s already exists", name)
	return nil
}
