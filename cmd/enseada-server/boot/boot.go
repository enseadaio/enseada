// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"context"
	"fmt"
	"strings"

	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/enseadaio/enseada/pkg/http"
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

type StartFunc func(ctx context.Context) error
type StopFunc func(ctx context.Context) error

func Boot(ctx context.Context) (StartFunc, StopFunc, error) {
	conf := conf()
	lvl := logLvl(conf)
	skb := []byte(conf.GetString("secret.key.base"))
	ph := conf.GetString("public.host")
	sec := conf.GetString("default.oauth.client.secret")
	oc := &goauth.Config{
		ClientID:     "enseada",
		ClientSecret: sec,
		Endpoint: goauth.Endpoint{
			AuthURL:   ph + "/oauth/authorize",
			TokenURL:  ph + "/oauth/token",
			AuthStyle: goauth.AuthStyleAutoDetect,
		},
		RedirectURL: ph + "/ui/callback",
		Scopes:      []string{"openid", "profile"},
	}

	data, err := dbClient(ctx, conf)
	if err != nil {
		return nil, nil, err
	}

	storage, err := storageBackend(conf)
	if err != nil {
		return nil, nil, err
	}

	echo, err := http.Boot(ctx, lvl, oc, skb)
	if err != nil {
		return nil, nil, err
	}

	authLogger := newLogger("auth", lvl)
	a, err := auth.Boot(ctx, echo, data, authLogger, skb, ph, sec)
	if err != nil {
		return nil, nil, err
	}

	mvnLogger := newLogger("maven2", lvl)
	if err := maven.Boot(ctx, mvnLogger, echo, data, storage, a.Enforcer, a.Store, a.Provider); err != nil {
		return nil, nil, err
	}

	return func(ctx context.Context) error {
			if err := a.Watcher.Start(ctx); err != nil {
				return err
			}

			port := conf.GetString("port")
			sslVar := conf.GetString("ssl")
			ssl := sslVar != "" && sslVar != "false" && sslVar != "no"

			address := fmt.Sprintf(":%s", port)
			if ssl {
				cert := conf.GetString("ssl.cert.path")
				key := conf.GetString("ssl.key.path")
				return echo.StartTLS(address, cert, key)
			} else {
				return echo.Start(address)
			}

		}, func(ctx context.Context) error {
			echo.Logger.Info("Shutting down server...")
			return echo.Shutdown(ctx)
		},
		nil
}

func conf() *viper.Viper {
	c := viper.NewWithOptions(
		viper.EnvKeyReplacer(strings.NewReplacer(".", "_")),
	)

	c.SetDefault("log.level", "info")
	c.SetDefault("port", "9623")
	c.SetDefault("storage.provider", "local")
	c.SetDefault("storage.dir", "uploads")
	c.SetDefault("root.password", "root")

	c.AutomaticEnv()
	return c
}
