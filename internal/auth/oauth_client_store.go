// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"errors"
	"fmt"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"
)

type OAuthClientStore struct {
	Logger log.Logger
	Data   *kivik.Client
}

func NewOAuthClientStore(data *kivik.Client, logger log.Logger) *OAuthClientStore {
	return &OAuthClientStore{Logger: logger, Data: data}
}

func (c *OAuthClientStore) GetClient(ctx context.Context, id string) (fosite.Client, error) {
	db := c.Data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)

	var client OAuthClient
	if err := row.ScanDoc(&client); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}
		c.Logger.Error(err)
		return nil, err
	}

	return &client, nil
}

func (c *OAuthClientStore) ListClients(ctx context.Context, selector couch.Query) ([]fosite.Client, error) {
	db := c.Data.DB(ctx, couch.OAuthDB)
	s := couch.Query{
		"kind": couch.KindOAuthClient,
	}
	if len(selector) > 0 {
		delete(selector, "kind")
		for k, v := range selector {
			s[k] = v
		}

	}

	rows, err := db.Find(ctx, couch.Query{
		"selector": s,
	})

	if err != nil {
		return nil, err
	}

	var clients []fosite.Client
	for rows.Next() {
		client := new(OAuthClient)
		if err := rows.ScanDoc(client); err != nil {
			return nil, err
		}
		clients = append(clients, client)
	}

	return clients, nil
}

func (c *OAuthClientStore) DeleteClient(ctx context.Context, id string) (fosite.Client, error) {
	db := c.Data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)
	cc := new(OAuthClient)
	if err := row.ScanDoc(cc); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}
		return nil, err
	}

	rev, err := db.Delete(ctx, cc.ID, cc.Rev)
	if err != nil {
		return nil, err
	}

	cc.Rev = rev
	return cc, nil
}

func (c *OAuthClientStore) SaveClient(ctx context.Context, client fosite.Client) error {
	cl, ok := client.(couch.Storable)
	if !ok {
		return errors.New(fmt.Sprintf("client %s does not implement couch.Storable", client.GetID()))
	}

	db := c.Data.DB(ctx, couch.OAuthDB)
	rev, err := db.Put(ctx, cl.GetID(), client)
	if err != nil {
		return err
	}

	cl.SetRev(rev)
	return nil
}

func (c *OAuthClientStore) InitDefaultClients(ctx context.Context, ph string, sec string) error {
	db := c.Data.DB(ctx, couch.OAuthDB)

	client, err := NewOAuthClient("enseada", sec,
		OAuthGrantTypes("authorization_code", "implicit", "refresh_token", "password", "client_credentials", "personal_access_token"),
		OAuthResponseTypes("code", "id_token", "token id_token", "code id_token", "code token", "code token id_token"),
		OAuthScopes("*"),
		OAuthRedirectURIs(ph+"/ui/callback"),
	)
	if err != nil {
		return err
	}

	err = c.initClient(ctx, db, client)
	if err != nil {
		return err
	}

	cli, err := NewOAuthClient("enseada-cli", "",
		OAuthGrantTypes("refresh_token", "password", "client_credentials", "personal_access_token"),
		OAuthResponseTypes("code", "id_token", "token id_token", "code id_token", "code token", "code token id_token"),
		OAuthScopes("*"),
		OAuthPublic(true),
	)
	if err != nil {
		return err
	}

	err = c.initClient(ctx, db, cli)
	if err != nil {
		return err
	}

	c.Logger.Debugf("created default OAuthProvider client. client_id: %s", "enseada")
	c.Logger.Debugf("created default OAuthProvider client. client_id: %s", "enseada-cli")
	return nil
}

func (c *OAuthClientStore) initClient(ctx context.Context, db *kivik.DB, cc *OAuthClient) error {
	_, rev, err := db.GetMeta(ctx, cc.GetID())
	if err != nil && kivik.StatusCode(err) != kivik.StatusNotFound {
		return err
	}

	cc.Rev = rev
	return c.SaveClient(ctx, cc)
}
