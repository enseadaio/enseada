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
	"syscall"
	"time"

	"github.com/enseadaio/enseada/cmd/enseada-server/boot"
	"github.com/joho/godotenv"
	"github.com/labstack/gommon/log"
)

func init() {
	if info, err := os.Stat("./.env"); err == nil && !info.IsDir() {
		err := godotenv.Load()
		if err != nil {
			panic(err)
		}
	}
}

func main() {
	l := log.New("main")
	bootctx, cancelboot := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancelboot()

	s := time.Now()
	start, stop, err := boot.Boot(bootctx)
	if err != nil {
		l.Fatal(err)
	}
	d := time.Since(s)
	l.Infof("Booted Enseada in %dms", d.Milliseconds())

	ctx, cancel := context.WithCancel(context.Background())
	go func() {
		if err := start(ctx); err != nil {
			l.Fatal(err)
		}
	}()

	quit := make(chan os.Signal)
	signal.Notify(quit, os.Interrupt, os.Kill, syscall.SIGTERM)
	<-quit

	cancel()

	shutdownctx, cancelshutdown := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancelshutdown()

	if err := stop(shutdownctx); err != nil {
		l.Fatal(err)
	}
}
