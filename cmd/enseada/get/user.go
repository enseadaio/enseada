// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package get

import (
	"context"
	"os"
	"time"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/jedib0t/go-pretty/table"

	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/twitchtv/twirp"
)

var user = &cobra.Command{
	Use:     "users [name]",
	Short:   "List users, or get a specific user",
	Aliases: []string{"user"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).UsersV1Beta1()

		if len(args) == 1 {
			if err := getUser(ctx, api, args[0]); err != nil {
				jww.ERROR.Fatal(err.Msg())
			}
		} else {
			if err := listUsers(ctx, api); err != nil {
				jww.ERROR.Fatal(err.Msg())
			}
		}
	},
}

func getUser(ctx context.Context, api authv1beta1.UsersAPI, username string) twirp.Error {
	res, err := api.GetUser(ctx, &authv1beta1.GetUserRequest{
		Username: username,
	})

	if err != nil {
		return err.(twirp.Error)
	}

	printUsers(res.User)
	return nil
}

func listUsers(ctx context.Context, client authv1beta1.UsersAPI) twirp.Error {
	res, err := client.ListUsers(ctx, &authv1beta1.ListUsersRequest{})
	if err != nil {
		err := err.(twirp.Error)
		if err.Code() == twirp.NotFound {
			printUsers()
			return nil

		}
		return err
	}

	printUsers(res.Users...)
	return nil
}

func printUsers(users ...*authv1beta1.User) {
	t := table.NewWriter()
	t.SetOutputMirror(os.Stdout)
	t.AppendHeader(table.Row{"Username"})
	for _, user := range users {
		t.AppendRow(table.Row{user.GetUsername()})
	}
	t.SetStyle(config.TableColorStyle)
	t.Render()
}
