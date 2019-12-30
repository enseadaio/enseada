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
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
)

type OAuthClientStore struct {
	Logger echo.Logger
	Data   *kivik.Client
}

func NewOAuthClientStore(data *kivik.Client, logger echo.Logger) *OAuthClientStore {
	return &OAuthClientStore{Logger: logger, Data: data}
}

func (c *OAuthClientStore) GetClient(ctx context.Context, id string) (fosite.Client, error) {
	log.Debugf("Getting client with id %s", id)
	db := c.Data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)

	var client OAuthClient
	if err := row.ScanDoc(&client); err != nil {
		log.Error(err)
		return nil, err
	}

	return &client, nil
}

func (c *OAuthClientStore) RegisterClient(ctx context.Context, client fosite.Client) error {
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

func (c *OAuthClientStore) InitDefaultClient(ctx context.Context, publicHost string, secret string) error {
	db := c.Data.DB(ctx, couch.OAuthDB)
	_, _, err := db.GetMeta(ctx, "enseada")
	if err == nil {
		return nil
	}
	if kivik.StatusCode(err) != kivik.StatusNotFound {
		return err
	}

	client, err := NewOAuthClient("enseada", secret,
		OAuthRedirectURIs(publicHost+"/ui/callback"),
		OAuthScopes("openid"),
	)
	if err != nil {
		return err
	}

	err = c.RegisterClient(ctx, client)
	if err != nil {
		return err
	}

	c.Logger.Infof("Created default OAuthProvider client. client_id: %s client_secret: %s", "enseada", secret)
	return nil
}
