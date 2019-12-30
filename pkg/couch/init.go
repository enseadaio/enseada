// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package couch

import (
	"context"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/couchdb"
	"github.com/go-kivik/kivik"
)

func NewClient(ctx context.Context, url string, user string, pwd string) (*kivik.Client, error) {
	client, err := kivik.New("couch", url)
	if err != nil {
		return nil, err
	}

	err = client.Authenticate(context.Background(), couchdb.BasicAuth(user, pwd))
	if err != nil {
		return nil, err
	}

	err = couch.Migrate(ctx, client)
	return client, err
}
