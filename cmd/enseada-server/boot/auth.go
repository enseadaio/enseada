// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"crypto/rand"
	"crypto/rsa"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
	"github.com/ory/fosite/compose"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

type AuthLogger echo.Logger

func authLog(lvl log.Lvl) AuthLogger {
	authLogger := log.New("auth")
	authLogger.SetLevel(lvl)
	return authLogger
}

func oauthProvider(store *auth.Store, skb enseada.SecretKeyBase) (fosite.OAuth2Provider, error) {
	key, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, err
	}

	return compose.ComposeAllEnabled(
		&compose.Config{},
		store,
		skb,
		key,
	), nil
}

type DefaultClientSecret string

func defaultClientSecret(conf *viper.Viper) DefaultClientSecret {
	return DefaultClientSecret(conf.GetString("default.oauth.client.secret"))
}

func oauthClient(host enseada.PublicHost, secret DefaultClientSecret) *goauth.Config {
	ph := string(host)
	return &goauth.Config{
		ClientID:     "enseada",
		ClientSecret: string(secret),
		Endpoint: goauth.Endpoint{
			AuthURL:   ph + "/oauth/authorize",
			TokenURL:  ph + "/oauth/token",
			AuthStyle: goauth.AuthStyleAutoDetect,
		},
		RedirectURL: ph + "/ui/callback",
		Scopes:      []string{"openid"},
	}
}

func newOAuthClientStore(data *kivik.Client, logger AuthLogger) *auth.OAuthClientStore {
	return auth.NewOAuthClientStore(data, logger)
}

func newOAuthRequestStore(data *kivik.Client, logger AuthLogger) *auth.OAuthRequestStore {
	return auth.NewOAuthRequestStore(data, logger)
}

func newOIDCSessionStore(data *kivik.Client, logger AuthLogger) *auth.OIDCSessionStore {
	return auth.NewOIDCSessionStore(data, logger)
}

func newPKCERequestStore(data *kivik.Client, logger AuthLogger) *auth.PKCERequestStore {
	return auth.NewPKCERequestStore(data, logger)
}

func newStore(data *kivik.Client, logger AuthLogger, cs *auth.OAuthClientStore, rs *auth.OAuthRequestStore, os *auth.OIDCSessionStore, ps *auth.PKCERequestStore) *auth.Store {
	return auth.NewStore(data, logger, cs, rs, os, ps)
}
