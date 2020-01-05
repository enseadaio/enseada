// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"context"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"cloud.google.com/go/errorreporting"
	"github.com/airbrake/gobrake/v4"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/enseadaio/enseada/pkg/errare"
	"github.com/enseadaio/enseada/pkg/errare/airbrake"
	logerr "github.com/enseadaio/enseada/pkg/errare/log"
	sentryerr "github.com/enseadaio/enseada/pkg/errare/sentry"
	"github.com/enseadaio/enseada/pkg/errare/stackdriver"
	"github.com/getsentry/sentry-go"

	"github.com/enseadaio/enseada/pkg/app"

	"github.com/enseadaio/enseada/pkg/log/adapters"
	"github.com/spf13/viper"

	"github.com/enseadaio/enseada/pkg/log"
	"github.com/joho/godotenv"
)

func init() {
	if info, err := os.Stat("./.env"); err == nil && !info.IsDir() {
		err := godotenv.Load()
		if err != nil {
			panic(err)
		}
	}
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
	c.SetDefault("sentry.environment", "development")
	c.SetDefault("airbrake.environment", "development")

	c.AutomaticEnv()
	return c
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

func main() {
	c := conf()
	lvl := log.Level(strings.ToLower(c.GetString("log.level")))
	l, err := adapters.NewZapLoggerAdapter(lvl)
	if err != nil {
		panic(err)
	}

	l.Info("Enseada booting up...")
	start := time.Now()

	errh, err := errorHandler(l, c)
	if err != nil {
		l.Fatal(err)
	}
	defer func() {
		if err := errh.Close(); err != nil {
			l.Error(err)
		}
	}()

	bootctx, cancelboot := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancelboot()

	mods, err := modules(bootctx, l, c, errh)
	if err != nil {
		l.Fatal(err)
	}

	a := app.New(
		app.Modules(mods...),
		app.OnError(func(err error) {
			errh.HandleError(err)
		}),
		app.OnPanic(func(v interface{}) {
			errh.HandlePanic(v)
		}),
	)

	ctx, cancel := context.WithCancel(context.Background())
	if err := a.Start(ctx); err != nil {
		l.Fatal(err)
	}
	l.Infof("started Enseada in %dms", time.Since(start).Milliseconds())

	quit := make(chan os.Signal)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGKILL, syscall.SIGTERM, syscall.SIGQUIT)
	<-quit

	l.Info("terminating Enseada...")
	defer l.Infof("stopped Enseada")

	cancel()

	shutdownctx, cancelshutdown := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancelshutdown()

	if err := a.Stop(shutdownctx); err != nil {
		l.Fatal(err)
	}
}
