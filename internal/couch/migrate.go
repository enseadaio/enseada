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
	"github.com/pkg/errors"
)

const (
	MavenDB = "maven2"
	OAuthDB = "oauth"
	UsersDB = "users"
	AclDB   = "acl"
)

func Transact(ctx context.Context, logger log.Logger, client *kivik.Client, f func(context.Context, log.Logger, *kivik.Client) error, dbname string) error {
	if err := f(ctx, logger, client); err != nil {
		e := client.DestroyDB(ctx, dbname)
		if e != nil {
			return errors.Wrap(err, e.Error())
		}
		return err
	}
	return nil
}
