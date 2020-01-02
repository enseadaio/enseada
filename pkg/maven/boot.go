// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"context"

	"github.com/uber-go/tally"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/casbin/casbin/v2"
	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

func Boot(ctx context.Context, logger log.Logger, e *echo.Echo, data *kivik.Client, store storage.Backend, enf *casbin.Enforcer, stats tally.Scope) error {
	mvn := maven.New(logger, data, store)
	mountRoutes(e, mvn, stats, enf)

	if err := couch.Transact(ctx, logger, data, migrateDB, couch.MavenDB); err != nil {
		return err
	}

	return nil
}

func migrateDB(ctx context.Context, logger log.Logger, data *kivik.Client) error {
	if err := couch.InitDb(ctx, logger, data, couch.MavenDB); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, data, couch.MavenDB, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, data, couch.MavenDB, "file_index", map[string]interface{}{
		"fields": []string{"files"},
	}); err != nil {
		return err
	}

	return nil
}
