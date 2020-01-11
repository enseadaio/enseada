// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package create

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/labstack/gommon/color"
	"github.com/spf13/viper"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/jedib0t/go-pretty/table"

	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/spf13/cobra"
	"github.com/twitchtv/twirp"
)

var mvnRepo = &cobra.Command{
	Use:     "mavenrepository [name]",
	Short:   "Create a new Maven repository",
	Aliases: []string{"mvnrepo", "mavenrepositories", "mvnrepos"},
	Args:    cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		_ = config.Client(ctx).MavenV1Beta1()

		gid := viper.GetString("groupID")
		aid := viper.GetString("artifactID")

		fmt.Printf("Creating repository %s:%s", color.Blue(gid), color.Blue(aid))
	},
}

func init() {
	mvnRepo.Flags().StringP("group-id", "g", "", "the new repo GroupID")
	mvnRepo.Flags().StringP("artifact-id", "a", "", "the new repo ArtifactID")

	viper.BindPFlag("groupID", mvnRepo.Flags().Lookup("group-id"))
	viper.BindPFlag("artifactID", mvnRepo.Flags().Lookup("artifact-id"))
}

func getRepo(ctx context.Context, api mavenv1beta1.MavenAPI, id string) twirp.Error {
	res, err := api.GetRepo(ctx, &mavenv1beta1.GetRepoRequest{
		Id: id,
	})

	if err != nil {
		return err.(twirp.Error)
	}

	printRepos(res.Repo)
	return nil
}

func listRepos(ctx context.Context, client mavenv1beta1.MavenAPI) twirp.Error {
	res, err := client.ListRepos(ctx, &mavenv1beta1.ListReposRequest{})
	if err != nil {
		return err.(twirp.Error)
	}

	printRepos(res.Repos...)
	return nil
}

func printRepos(repos ...*mavenv1beta1.Repo) {
	t := table.NewWriter()
	t.SetOutputMirror(os.Stdout)
	t.AppendHeader(table.Row{"Name", "Group Username", "Artifact Username"})
	for _, repo := range repos {
		t.AppendRow(table.Row{repo.GetId(), repo.GetGroupId(), repo.GetArtifactId()})
	}
	t.SetStyle(table.Style{
		Name:    "StyleColoredSuperDark",
		Box:     table.StyleBoxDefault,
		Color:   table.ColorOptionsDark,
		Format:  table.FormatOptionsDefault,
		Options: table.OptionsNoBordersAndSeparators,
		Title:   table.TitleOptionsDark,
	})
	t.Render()
}
