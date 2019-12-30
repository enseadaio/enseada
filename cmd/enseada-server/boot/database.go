// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"context"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/go-kivik/kivik"
	"github.com/spf13/viper"
)

func dbClient(ctx context.Context, conf *viper.Viper) (*kivik.Client, error) {
	url := conf.GetString("couchdb.url")
	user := conf.GetString("couchdb.user")
	pwd := conf.GetString("couchdb.password")

	return couch.NewClient(ctx, url, user, pwd)
}
