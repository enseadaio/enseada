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

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"

	"github.com/twitchtv/twirp"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/labstack/gommon/color"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var user = &cobra.Command{
	Use:     "user [username]",
	Short:   "Delete a user",
	Aliases: []string{"users"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).UsersV1Beta1()

		uid := args[0]

		res, err := api.DeleteUser(ctx, &authv1beta1.DeleteUserRequest{
			Username: uid,
		})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		repo := res.GetUser()
		fmt.Printf("Deleted user %s", color.Blue(repo.GetUsername()))
		fmt.Println()
	},
}
