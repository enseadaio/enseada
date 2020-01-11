// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package get

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/jedib0t/go-pretty/table"

	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/spf13/cobra"
	"github.com/twitchtv/twirp"
)

var mvnRepo = &cobra.Command{
	Use:     "mavenrepository [name]",
	Short:   "Get a Maven repository",
	Aliases: []string{"mvnrepo", "mavenrepositories", "mvnrepos"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).MavenV1Beta1()

		if len(args) == 1 {
			if err := getRepo(ctx, api, args[0]); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		} else {
			if err := listRepos(ctx, api); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		}
	},
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
	t.SetStyle(config.TableColorStyle)
	t.Render()
}
