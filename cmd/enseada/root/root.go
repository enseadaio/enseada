// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package root

import (
	"path/filepath"
	"strings"

	"github.com/enseadaio/enseada/cmd/enseada/create"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	del "github.com/enseadaio/enseada/cmd/enseada/delete"
	"github.com/enseadaio/enseada/cmd/enseada/get"
	"github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"
)

var rootCmd = &cobra.Command{
	Use:   "enseada",
	Short: "A Cloud native multi-package registry",
	Long: `A Cloud native multi-package registry

Enseada is a modern, fast and scalable package registry, designed from the ground up to run in elastic, container-based environments and to be highly available and distributed.
More information available at https://enseada.io`,
}

func init() {
	jww.SetStdoutThreshold(jww.LevelTrace)
	jww.SetLogThreshold(jww.LevelDebug)

	viper.SetEnvPrefix("enseada")
	viper.AutomaticEnv()
	viper.EnvKeyReplacer(strings.NewReplacer(".", "_"))

	// Find home directory.
	home, err := homedir.Dir()
	if err != nil {
		jww.ERROR.Fatal(err)
	}
	def := filepath.Join(home, ".config", "enseada", "config.hcl")

	cobra.OnInitialize(config.InitConfigFile)
	cobra.OnInitialize(config.ReadConfigFile)

	// Flags
	rootCmd.PersistentFlags().StringP("config", "c", def, "config file to use")
	rootCmd.PersistentFlags().StringP("profile", "p", "default", "profile used for authentication")

	viper.BindPFlag("config", rootCmd.PersistentFlags().Lookup("config"))
	viper.BindPFlag("profile", rootCmd.PersistentFlags().Lookup("profile"))

	// Commands
	rootCmd.AddCommand(versionCmd)
	rootCmd.AddCommand(loginCmd)
	rootCmd.AddCommand(get.RootCmd)
	rootCmd.AddCommand(create.RootCmd)
	rootCmd.AddCommand(del.RootCmd)
}

func Execute() error {
	return rootCmd.Execute()
}
