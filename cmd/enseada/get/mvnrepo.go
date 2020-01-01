// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package get

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"os"
	"time"

	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/jedib0t/go-pretty/table"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/twitchtv/twirp"
)

var getMvnRepo = &cobra.Command{
	Use:     "mavenrepository [name]",
	Short:   "Get a Maven repository",
	Aliases: []string{"mvnrepo", "mavenrepositories", "mvnrepos"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		url := viper.GetString("url")
		client := mavenv1beta1.NewMavenAPIProtobufClient(url, &http.Client{})
		ctx, cancel := context.WithTimeout(context.Background(), time.Second*15)
		defer cancel()

		if len(args) == 1 {
			if err := getRepo(ctx, client, args[0]); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		} else {
			if err := listRepos(ctx, client); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		}
	},
}

func getRepo(ctx context.Context, client mavenv1beta1.MavenAPI, id string) error {
	res, err := client.GetRepo(ctx, &mavenv1beta1.GetRepoRequest{
		Id: id,
	})

	if err != nil {
		twerr := err.(twirp.Error)
		return errors.New(twerr.Msg())
	}

	printRepos(res.Repo)
	return nil
}

func listRepos(ctx context.Context, client mavenv1beta1.MavenAPI) error {
	res, err := client.ListRepos(ctx, &mavenv1beta1.ListReposRequest{})
	if err != nil {
		twerr := err.(twirp.Error)
		return errors.New(twerr.Msg())
	}

	printRepos(res.Repos...)
	return nil
}

func printRepos(repos ...*mavenv1beta1.Repo) {
	t := table.NewWriter()
	t.SetOutputMirror(os.Stdout)
	t.AppendHeader(table.Row{"Name", "Group ID", "Artifact ID"})
	for _, repo := range repos {
		t.AppendRow(table.Row{repo.GetId(), repo.GetGroupId(), repo.GetArtifactId()})
	}
	t.Render()
}
