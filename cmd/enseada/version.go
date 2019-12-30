// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package main

import (
	"fmt"

	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/spf13/cobra"
)

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print enseada version and exit",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("%d.%d.%d%s", enseada.VersionMajor, enseada.VersionMinor, enseada.VersionPatch, enseada.VersionSuffix)
	},
}
