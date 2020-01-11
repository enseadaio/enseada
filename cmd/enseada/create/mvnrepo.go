// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package create

import (
	"context"
	"fmt"
	"time"

	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/twitchtv/twirp"

	"github.com/labstack/gommon/color"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/spf13/cobra"
)

var mvnRepo = &cobra.Command{
	Use:     "mavenrepository [name]",
	Short:   "Create a new Maven repository",
	Aliases: []string{"mvnrepo", "mavenrepositories", "mvnrepos"},
	Args:    cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).MavenV1Beta1()

		gid := viper.GetString("groupID")
		aid := viper.GetString("artifactID")

		if gid == "" {
			jww.ERROR.Fatal("GroupID is required")
		}

		if aid == "" {
			jww.ERROR.Fatal("ArtifactID is required")
		}

		res, err := api.CreateRepo(ctx, &mavenv1beta1.CreateRepoRequest{
			GroupId:    gid,
			ArtifactId: aid,
		})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		repo := res.GetRepo()
		fmt.Printf("Created repository %s", color.Blue(repo.GetId()))
		fmt.Println()
	},
}

func init() {
	mvnRepo.Flags().StringP("group-id", "g", "", "the new repo GroupID")
	mvnRepo.Flags().StringP("artifact-id", "a", "", "the new repo ArtifactID")

	viper.BindPFlag("groupID", mvnRepo.Flags().Lookup("group-id"))
	viper.BindPFlag("artifactID", mvnRepo.Flags().Lookup("artifact-id"))
}
