// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"github.com/enseadaio/enseada/cmd/enseada/get"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var cfgFile string

var rootCmd = &cobra.Command{
	Use:   "enseada",
	Short: "A Cloud native multi-package registry",
	Long: `A Cloud native multi-package registry

Enseada is a modern, fast and scalable package registry, designed from the ground up to run in elastic, container-based environments and to be highly available and distributed.
More information available at https://enseada.io`,
}

func init() {
	cobra.OnInitialize(initConfig)
	// Commands
	rootCmd.AddCommand(versionCmd)
	rootCmd.AddCommand(get.RootCmd)
}

func initConfig() {
	/*if cfgFile != "" {
		if !(strings.HasSuffix(cfgFile, ".yaml") || strings.HasSuffix(cfgFile, ".yml")) {
			fmt.Print("Config file must be a YAML file (with extension .yml or .yaml)")
			os.Exit(1)
		}
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		// Find home directory.
		home, err := homedir.Dir()
		if err != nil {
			fmt.Print(err)
			os.Exit(1)
		}

		configPath := filepath.Join(home, ".config/enseada")
		viper.AddConfigPath(configPath)
		viper.SetConfigName("config.yml")
	}

	viper.SetConfigType("yaml")*/
	viper.AutomaticEnv()

	viper.SetDefault("url", "http://localhost:9623")

	//if err := viper.ReadInConfig(); err != nil {
	//	fmt.Print(err)
	//	os.Exit(1)
	//}
}
