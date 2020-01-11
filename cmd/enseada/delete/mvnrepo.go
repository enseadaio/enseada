// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package delete

import (
	"context"
	"fmt"
	"time"

	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/twitchtv/twirp"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/labstack/gommon/color"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var mvnRepo = &cobra.Command{
	Use:     "mavenrepository [name]",
	Short:   "Delete a Maven repository",
	Aliases: []string{"mvnrepo", "mavenrepositories", "mvnrepos"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).MavenV1Beta1()
		id := args[0]

		res, err := api.DeleteRepo(ctx, &mavenv1beta1.DeleteRepoRequest{
			Id: id,
		})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		repo := res.GetRepo()
		fmt.Printf("Deleted repository %s", color.Blue(repo.GetId()))
	},
}
