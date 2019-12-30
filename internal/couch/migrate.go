// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package couch

import (
	"context"
	"github.com/go-kivik/kivik"
	"github.com/pkg/errors"
)

const (
	MavenDB = "maven2"
	OAuthDB = "oauth"
	UsersDB = "users"
	AclDB   = "acl"
)

func Migrate(ctx context.Context, client *kivik.Client) error {
	if err := transact(ctx, client, maven, MavenDB); err != nil {
		return err
	}

	if err := transact(ctx, client, oauth, OAuthDB); err != nil {
		return err
	}

	if err := transact(ctx, client, users, UsersDB); err != nil {
		return err
	}

	if err := transact(ctx, client, acl, AclDB); err != nil {
		return err
	}

	return nil
}

func transact(ctx context.Context, client *kivik.Client, f func(context.Context, *kivik.Client) error, dbname string) error {
	if err := f(ctx, client); err != nil {
		e := client.DestroyDB(ctx, dbname)
		if e != nil {
			return errors.Wrap(err, e.Error())
		}
		return err
	}
	return nil
}

func maven(ctx context.Context, client *kivik.Client) error {

	if err := InitDb(ctx, client, MavenDB); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, MavenDB, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, MavenDB, "file_index", map[string]interface{}{
		"fields": []string{"files"},
	}); err != nil {
		return err
	}

	return nil
}

func oauth(ctx context.Context, client *kivik.Client) error {
	if err := InitDb(ctx, client, OAuthDB); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "oauth_reqs_index", Query{
		"fields": []string{"req.id"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "oauth_sigs_index", Query{
		"fields": []string{"sig"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "openid_reqs_index", Query{
		"fields": []string{"auth_code"},
	}); err != nil {
		return err
	}

	return nil
}

func users(ctx context.Context, client *kivik.Client) error {
	if err := InitDb(ctx, client, UsersDB); err != nil {
		return err
	}

	return nil
}

func acl(ctx context.Context, client *kivik.Client) error {
	if err := InitDb(ctx, client, AclDB); err != nil {
		return err
	}

	return nil
}
