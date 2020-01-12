// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package create

import (
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var client = &cobra.Command{
	Use:     "client",
	Short:   "Create a new OAuth client",
	Aliases: []string{"clients"},
	//Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		jww.WARN.Fatal("not yet implemented")
	},
}
