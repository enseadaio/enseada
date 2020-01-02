// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"crypto/rand"
	"crypto/rsa"

	"github.com/uber-go/tally"

	"github.com/enseadaio/enseada/pkg/log"

	rice "github.com/GeertJohan/go.rice"
	"github.com/casbin/casbin/v2"
	"github.com/casbin/casbin/v2/model"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/ory/fosite/compose"
)

type Components struct {
	Store    *auth.Store
	Enforcer *casbin.Enforcer
	Watcher  *CasbinWatcher
	Provider fosite.OAuth2Provider
}

func Boot(ctx context.Context, logger log.Logger, e *echo.Echo, data *kivik.Client, stats tally.Scope, skb []byte, ph string, clientSecret string) (*Components, error) {
	if err := couch.Transact(ctx, logger, data, migrateAclDb, couch.AclDB); err != nil {
		return nil, err
	}

	if err := couch.Transact(ctx, logger, data, migrateOAuthDb, couch.OAuthDB); err != nil {
		return nil, err
	}

	if err := couch.Transact(ctx, logger, data, migrateUsersDb, couch.UsersDB); err != nil {
		return nil, err
	}

	s := createStore(data, logger)

	enf, w, err := createCasbin(data, logger)
	if err != nil {
		return nil, err
	}

	key, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, err
	}

	op := compose.ComposeAllEnabled(
		&compose.Config{},
		s,
		skb,
		key,
	)

	err = s.InitDefaultClients(ctx, ph, clientSecret)
	if err != nil {
		return nil, err
	}

	root := auth.RootUser("root")
	if err := s.SaveUser(ctx, root); err != nil && kivik.StatusCode(err) != kivik.StatusConflict {
		return nil, err
	}

	mountRoutes(ctx, e, s, op, enf, stats, middleware.Session(skb))
	return &Components{
		Store:    s,
		Enforcer: enf,
		Watcher:  w,
		Provider: op,
	}, nil
}

func createStore(data *kivik.Client, logger log.Logger) *auth.Store {
	oAuthClientStore := auth.NewOAuthClientStore(data, logger)
	oAuthRequestStore := auth.NewOAuthRequestStore(data, logger)
	oidcSessionStore := auth.NewOIDCSessionStore(data, logger)
	pkceRequestStore := auth.NewPKCERequestStore(data, logger)
	userStore := auth.NewUserStore(data, logger)
	return auth.NewStore(data, logger, oAuthClientStore, oAuthRequestStore, oidcSessionStore, pkceRequestStore, userStore)
}

func createCasbin(data *kivik.Client, logger log.Logger) (*casbin.Enforcer, *CasbinWatcher, error) {
	box := rice.MustFindBox("../../conf/")
	m, err := model.NewModelFromString(box.MustString("casbin_model.conf"))
	if err != nil {
		return nil, nil, err
	}

	adapter, err := NewCasbinAdapter(data, logger)
	if err != nil {
		return nil, nil, err
	}

	watcher := NewCasbinWatcher(data, logger)

	e, err := casbin.NewEnforcer(m, adapter)
	if err != nil {
		return nil, nil, err
	}

	e.EnableLog(false)
	e.EnableAutoSave(true)

	err = e.SetWatcher(watcher)
	if err != nil {
		return nil, nil, err
	}

	return e, watcher, nil
}

func migrateAclDb(ctx context.Context, logger log.Logger, client *kivik.Client) error {
	if err := couch.InitDb(ctx, logger, client, couch.AclDB); err != nil {
		return err
	}

	return nil
}

func migrateOAuthDb(ctx context.Context, logger log.Logger, client *kivik.Client) error {
	if err := couch.InitDb(ctx, logger, client, couch.OAuthDB); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, client, couch.OAuthDB, "kind_index", couch.Query{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, client, couch.OAuthDB, "oauth_reqs_index", couch.Query{
		"fields": []string{"req.id"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, client, couch.OAuthDB, "oauth_sigs_index", couch.Query{
		"fields": []string{"sig"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, logger, client, couch.OAuthDB, "openid_reqs_index", couch.Query{
		"fields": []string{"auth_code"},
	}); err != nil {
		return err
	}

	return nil
}

func migrateUsersDb(ctx context.Context, logger log.Logger, client *kivik.Client) error {
	if err := couch.InitDb(ctx, logger, client, couch.UsersDB); err != nil {
		return err
	}

	return nil
}
