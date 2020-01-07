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
	"github.com/joho/godotenv"
	"github.com/spf13/cobra"

	"github.com/spf13/viper"

	"github.com/enseadaio/enseada/pkg/log"
)

const (
	defaultPort = 9623
)

var c = viper.NewWithOptions(
	viper.EnvKeyReplacer(strings.NewReplacer(".", "_")),
)

var rootCmd = &cobra.Command{
	Use:   "enseada-server",
	Short: "A Cloud native multi-package registry",
	Long: `A Cloud native multi-package registry
	
Enseada is a modern, fast and scalable package registry, designed from the ground up to run in elastic, container-based environments and to be highly available and distributed.
More information available at https://enseada.io`,
	PreRunE: func(cmd *cobra.Command, args []string) error {
		if info, err := os.Stat("./.env"); err == nil && !info.IsDir() {
			err := godotenv.Load()
			if err != nil {
				return err
			}
		}

		c.SetDefault("log.level", "info")
		c.SetDefault("port", defaultPort)
		c.SetDefault("storage.provider", "local")
		c.SetDefault("local.storage.dir", "uploads")
		c.SetDefault("root.password", "root")
		c.SetDefault("sentry.environment", "development")
		c.SetDefault("airbrake.environment", "development")

		c.RegisterAlias("aws.region", "s3.region")
		c.RegisterAlias("aws.bucket", "s3.bucket")
		c.RegisterAlias("aws.bucket.prefix", "s3.bucket.prefix")
		c.RegisterAlias("aws.s3.endpoint", "s3.endpoint")
		c.RegisterAlias("aws.s3.sse", "s3.sse")

		c.AutomaticEnv()

		return nil
	},
	Run: func(cmd *cobra.Command, args []string) {
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
	},
}

func init() {
	// Subcommands
	rootCmd.AddCommand(versionCmd)

	// Flags
	rootCmd.Flags().IntP("port", "p", defaultPort, "HTTP port to use")
	rootCmd.Flags().StringP("log", "l", string(log.INFO), "logging level )")

	c.BindPFlag("port", rootCmd.Flags().Lookup("port"))
	c.BindPFlag("log.level", rootCmd.Flags().Lookup("log"))
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		panic(err)
	}
}
