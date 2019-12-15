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
