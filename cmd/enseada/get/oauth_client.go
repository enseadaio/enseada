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
	"strings"
	"time"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/jedib0t/go-pretty/table"

	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/twitchtv/twirp"
)

var client = &cobra.Command{
	Use:     "client [id]",
	Short:   "List OAuth clients, or get a specific client",
	Aliases: []string{"clients"},
	Args:    cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).OAuthClientsV1Beta1()

		if len(args) == 1 {
			if err := getClient(ctx, api, args[0]); err != nil {
				jww.ERROR.Fatal(err.Msg())
			}
		} else {
			if err := listClient(ctx, api); err != nil {
				jww.ERROR.Fatal(err.Msg())
			}
		}
	},
}

func getClient(ctx context.Context, api authv1beta1.OAuthClientsAPI, id string) twirp.Error {
	res, err := api.GetClient(ctx, &authv1beta1.GetClientRequest{
		Id: id,
	})

	if err != nil {
		return err.(twirp.Error)
	}

	printClient(res.GetClient())
	return nil
}

func listClient(ctx context.Context, client authv1beta1.OAuthClientsAPI) twirp.Error {
	res, err := client.ListClients(ctx, &authv1beta1.ListClientsRequest{})
	if err != nil {
		err := err.(twirp.Error)
		if err.Code() == twirp.NotFound {
			printClients()
			return nil

		}
		return err
	}

	printClients(res.GetClients()...)
	return nil
}

func printClient(client *authv1beta1.OAuthClient) {
	t := table.NewWriter()
	t.SetOutputMirror(os.Stdout)
	t.AppendHeader(table.Row{
		"ID",
		"Active",
		"Public",
		"Scopes",
		"Audiences",
		"Grant Types",
		"Response Types",
		"Redirect URIs",
	})
	t.AppendRow(table.Row{
		client.GetId(),
		true,
		client.GetPublic(),
		csv(client.GetScopes()),
		csv(client.GetAudiences()),
		csv(client.GetGrantTypes()),
		csv(client.GetResponseTypes()),
		csv(client.GetRedirectUris()),
	})
	t.SetStyle(config.TableColorStyle)
	t.Render()
}

func printClients(clients ...*authv1beta1.OAuthClient) {
	t := table.NewWriter()
	t.SetOutputMirror(os.Stdout)
	t.AppendHeader(table.Row{
		"ID",
		"Active",
		"Public",
		"Scopes",
	})
	for _, client := range clients {
		t.AppendRow(table.Row{
			client.GetId(),
			true,
			client.GetPublic(),
			csv(client.GetScopes()),
		})
	}
	t.SetStyle(config.TableColorStyle)
	t.Render()
}

func csv(els []string) string {
	s := make([]string, len(els))
	for i, el := range els {
		s[i] = fmt.Sprintf("%v", el)
	}
	cs := strings.Join(s, "\n")
	return cs
}
