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

	c.AutomaticEnv()
	return c
}

func main() {
	c := conf()
	lvl := log.Level(strings.ToLower(c.GetString("log.level")))
	l, err := adapters.NewZapLoggerAdapter(lvl)
	if err != nil {
		panic(err)
	}

	mods, err := modules(l, c)
	if err != nil {
		l.Fatal(err)
	}

	a := app.New(
		app.Modules(mods...),
		app.OnError(func(err error) {
			l.Error(err)
		}),
		app.OnPanic(func(v interface{}) {
			l.Fatalf("panic: %v", v)
		}),
	)

	start := time.Now()
	ctx, cancel := context.WithCancel(context.Background())
	if err := a.Start(ctx); err != nil {
		l.Fatal(err)
	}
	l.Info("started Enseada in %dms", time.Since(start).Milliseconds())

	quit := make(chan os.Signal)
	signal.Notify(quit, os.Interrupt, os.Kill, syscall.SIGTERM)
	<-quit

	cancel()

	shutdownctx, cancelshutdown := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancelshutdown()

	if err := a.Stop(shutdownctx); err != nil {
		l.Fatal(err)
	}
}
