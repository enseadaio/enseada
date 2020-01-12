// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package edit

import (
	"context"
	"fmt"
	"time"

	"github.com/labstack/gommon/color"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"

	"github.com/twitchtv/twirp"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var client = &cobra.Command{
	Use:     "client [id]",
	Short:   "Edit an OAuth client",
	Aliases: []string{"clients"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).OAuthClientsV1Beta1()

		id := args[0]

		gres, err := api.GetClient(ctx, &authv1beta1.GetClientRequest{
			Id: id,
		})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		client := gres.GetClient()
		changed, err := OpenInEditor(client)
		if err != nil {
			jww.ERROR.Fatal(err)
		}

		if changed {
			ures, err := api.UpdateClient(ctx, &authv1beta1.UpdateClientRequest{
				Client: client,
			})

			if err != nil {
				jww.ERROR.Fatal(err)
			}

			fmt.Printf("Edited client %s", color.Blue(ures.GetClient().GetId()))
			fmt.Println()
		} else {
			fmt.Println("Edit cancelled, no changes made")
		}
	},
}
