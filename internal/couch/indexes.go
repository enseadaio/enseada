// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package couch

import (
	"context"
	"fmt"

	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/kivik"
)

func InitIndex(ctx context.Context, logger log.Logger, client *kivik.Client, dbName string, name string, idx map[string]interface{}) error {
	db := client.DB(ctx, dbName)
	logger.Debugf("initializing index %s on db %s", name, dbName)
	return db.CreateIndex(ctx, fmt.Sprintf("%s_idx", name), name, idx)
}
