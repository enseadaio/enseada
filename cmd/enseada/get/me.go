// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package get

import (
	"context"
	"time"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/twitchtv/twirp"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var me = &cobra.Command{
	Use:   "me",
	Short: "Get the currently authenticated user",
	Args:  cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).UsersV1Beta1()
		res, err := api.GetCurrentUser(ctx, &authv1beta1.GetCurrentUserRequest{})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		user := res.GetUser()
		printUsers(user)
	},
}
