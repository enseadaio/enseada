// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"context"
	"errors"
	"strings"

	"cloud.google.com/go/errorreporting"
	"github.com/airbrake/gobrake/v4"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/enseadaio/enseada/pkg/errare/airbrake"
	logerr "github.com/enseadaio/enseada/pkg/errare/log"
	sentryerr "github.com/enseadaio/enseada/pkg/errare/sentry"
	"github.com/enseadaio/enseada/pkg/errare/stackdriver"
	"github.com/getsentry/sentry-go"

	"github.com/enseadaio/enseada/pkg/observability"

	"github.com/enseadaio/enseada/pkg/errare"

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

func modules(ctx context.Context, logger log.Logger, conf *viper.Viper, errh errare.Handler) ([]app.Module, error) {
	skb := []byte(conf.GetString("secret.key.base"))
	if skb == nil {
		return nil, errors.New("no secret key base configured")
	}

	ph := conf.GetString("public.host")
	if ph == "" {
		return nil, errors.New("no public host configured")
	}

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

	sm, err := storage.NewModule(ctx, logger.Child("storage"), storageConfig(conf))

	sslVar := conf.GetString("ssl")
	ssl := sslVar != "" && sslVar != "false" && sslVar != "no"
	var tls *http.TLSConfig = nil
	if ssl {
		tls = &http.TLSConfig{
			KeyFile:  conf.GetString("ssl.key.path"),
			CertFile: conf.GetString("ssl.cert.path"),
		}
	}
	hm, err := http.NewModule(ctx, logger.Child("http"), errh, oc, skb, conf.GetInt("port"), tls)
	if err != nil {
		return nil, err
	}

	om, err := observability.NewModule(logger.Child("observability"), hm.Echo, errh)
	if err != nil {
		return nil, err
	}

	am, err := auth.NewModule(ctx, logger.Child("auth"), dm.Data, hm.Echo, errh, om.Registry, skb, ph, conf.GetString("root.password"))
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
		om,
		am,
		mm,
	}, nil
}

func storageConfig(c *viper.Viper) storage.Config {
	return storage.Config{
		Provider: c.GetString("storage.provider"),
		Local: storage.LocalConfig{
			StorageDir: c.GetString("local.storage.dir"),
		},
		S3: storage.S3Config{
			Bucket:   c.GetString("s3.bucket"),
			Prefix:   c.GetString("s3.bucket.prefix"),
			Region:   c.GetString("s3.region"),
			Endpoint: c.GetString("s3.endpoint"),
			SSE:      c.GetString("s3.sse"),
		},
		Azure: storage.AzureConfig{
			Bucket: c.GetString("azure.bucket"),
			Prefix: c.GetString("azure.bucket.prefix"),
		},
		GCS: storage.GCSConfig{
			Bucket: c.GetString("gcs.bucket"),
			Prefix: c.GetString("gcs.bucket.prefix"),
		},
	}
}

func errorHandler(logger log.Logger, c *viper.Viper) (errare.Handler, error) {
	l := logerr.NewHandler(logger, true)

	var erh errare.Handler
	reporter := strings.ToLower(c.GetString("error.reporter"))
	switch reporter {
	case "sentry":
		sen, err := sentryerr.NewHandler(sentry.ClientOptions{
			Dsn:              c.GetString("sentry.dsn"),
			Debug:            false,
			AttachStacktrace: true,
			Environment:      c.GetString("sentry.environment"),
		})
		if err != nil {
			return nil, err
		}
		erh = sen
	case "stackdriver":
		sd, err := stackdriver.NewHandler(context.Background(), c.GetString("google.project.id"), errorreporting.Config{
			ServiceName:    "enseada",
			ServiceVersion: enseada.VersionString(),
			OnError: func(err error) {
				l.HandleError(err)
			},
		})
		if err != nil {
			return nil, err
		}
		erh = sd
	case "airbrake":
		erh = airbrake.NewHandler(&gobrake.NotifierOptions{
			ProjectId:   c.GetInt64("airbrake.project.id"),
			ProjectKey:  c.GetString("airbrake.project.key"),
			Environment: c.GetString("airbrake.environment"),
		})
	default:
		logger.Info("no error reporter configured. Defaulting to log")
		return l, nil
	}

	logger.Infof("configured error reporting with %s", reporter)
	return errare.Compose(l, erh), nil
}
