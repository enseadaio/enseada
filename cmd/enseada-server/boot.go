// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"context"

	"github.com/enseadaio/enseada/pkg/app"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/enseadaio/enseada/pkg/http"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/enseadaio/enseada/pkg/storage"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

func modules(ctx context.Context, logger log.Logger, conf *viper.Viper) ([]app.Module, error) {
	skb := []byte(conf.GetString("secret.key.base"))
	ph := conf.GetString("public.host")
	oc := &goauth.Config{
		ClientID: "enseada",
		Endpoint: goauth.Endpoint{
			AuthURL:   ph + "/oauth/authorize",
			TokenURL:  ph + "/oauth/token",
			AuthStyle: goauth.AuthStyleAutoDetect,
		},
		RedirectURL: ph + "/ui/callback",
		Scopes:      []string{"openid", "profile"},
	}

	dm, err := couch.NewModule(
		ctx,
		logger.Child("couch"),
		conf.GetString("couchdb.url"),
		conf.GetString("couchdb.user"),
		conf.GetString("couchdb.password"),
	)
	if err != nil {
		return nil, err
	}

	sm, err := storage.NewModule(ctx, logger.Child("storage"), storage.Config{
		Provider:   conf.GetString("storage.provider"),
		StorageDir: conf.GetString("storage.dir"),
	})

	sslVar := conf.GetString("ssl")
	ssl := sslVar != "" && sslVar != "false" && sslVar != "no"
	var tls *http.TLSConfig = nil
	if ssl {
		tls = &http.TLSConfig{
			KeyFile:  conf.GetString("ssl.key.path"),
			CertFile: conf.GetString("ssl.cert.path"),
		}
	}
	hm, err := http.NewModule(ctx, logger.Child("http"), oc, skb, conf.GetInt("port"), tls)
	if err != nil {
		return nil, err
	}

	am, err := auth.NewModule(ctx, logger.Child("auth"), dm.Data, hm.Echo, skb, ph, conf.GetString("root.password"))
	if err != nil {
		return nil, err
	}

	mm, err := maven.NewModule(ctx, logger.Child("maven"), hm.Echo, dm.Data, sm.Backend, am.Enforcer, am.Store, am.Provider)
	if err != nil {
		return nil, err
	}

	return []app.Module{
		dm,
		sm,
		hm,
		am,
		mm,
	}, nil
}
