// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"context"
	"fmt"

	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/enseadaio/enseada/pkg/observability"

	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/enseadaio/enseada/pkg/http"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

func Boot(ctx context.Context, logger log.Logger, conf *viper.Viper) (enseada.StartFunc, enseada.StopFunc, error) {
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

	echo, stopHttp, err := http.Boot(ctx, logger.Child("echo"), oc, skb)
	if err != nil {
		return nil, nil, err
	}

	stopObs, err := observability.Boot(ctx, echo)
	if err != nil {
		return nil, nil, err
	}

	a, err := auth.Boot(ctx, echo, data, logger.Child("auth"), skb, ph, sec)
	if err != nil {
		return nil, nil, err
	}

	if err := maven.Boot(ctx, logger.Child("maven2"), echo, data, storage, a.Enforcer, a.Store, a.Provider); err != nil {
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
			if err := stopHttp(ctx); err != nil {
				return err
			}

			if err := stopObs(ctx); err != nil {
				return err
			}
			return nil
		},
		nil
}
