// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/enseadaio/enseada/cmd/enseada-server/boot"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/joho/godotenv"
	"github.com/labstack/gommon/log"
	"github.com/spf13/viper"
)

func init() {
	if info, err := os.Stat("./.env"); err == nil && !info.IsDir() {
		err := godotenv.Load()
		if err != nil {
			panic(err)
		}
	}
}

func conf() (*viper.Viper, error) {
	c := viper.NewWithOptions(
		viper.EnvKeyReplacer(strings.NewReplacer(".", "_")),
	)

	c.SetDefault("log.level", "info")
	c.SetDefault("port", "9623")
	c.SetDefault("storage.provider", "local")
	c.SetDefault("storage.dir", "uploads")
	c.SetDefault("root.password", "root")

	c.AutomaticEnv()
	return c, nil
}

func run(srv *enseada.Server, conf *viper.Viper) error {
	port := conf.GetString("port")
	sslVar := conf.GetString("ssl")
	ssl := sslVar != "" && sslVar != "false" && sslVar != "no"

	address := fmt.Sprintf(":%s", port)
	if ssl {
		cert := conf.GetString("ssl.cert.path")
		key := conf.GetString("ssl.key.path")
		return srv.StartTLS(address, cert, key)
	} else {
		return srv.Start(address)
	}
}

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	c, err := conf()
	exitOnErr(err)

	srv, err := boot.Boot(ctx, c)
	exitOnErr(err)

	err = run(srv, c)
	exitOnErr(err)
}

func exitOnErr(err error) {
	if err != nil {
		log.Fatal(err)
	}
}
