// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package create

import (
	"bytes"
	"context"
	"fmt"
	"io/ioutil"
	"os"
	"strings"
	"time"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"golang.org/x/crypto/ssh/terminal"

	"github.com/twitchtv/twirp"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/labstack/gommon/color"
	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
)

var user = &cobra.Command{
	Use:     "user [username]",
	Short:   "Create a new user",
	Aliases: []string{"users"},
	Args:    cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		rs, err := cmd.Flags().GetBool("password-stdin")
		if err != nil {
			jww.ERROR.Fatal(err)
		}

		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		defer cancel()

		api := config.Client(ctx).UsersV1Beta1()

		uid := args[0]
		var pwd []byte
		if rs {
			p, err := ioutil.ReadAll(os.Stdin)
			if err != nil {
				jww.ERROR.Fatal(err)
			}

			pwd = p
		} else {
			fmt.Print("New user's password: ")
			p, err := terminal.ReadPassword(int(os.Stdin.Fd()))
			if err != nil {
				jww.ERROR.Fatal(err)
			}
			fmt.Println()

			fmt.Print("Confirm password: ")
			cp, err := terminal.ReadPassword(int(os.Stdin.Fd()))
			if err != nil {
				jww.ERROR.Fatal(err)
			}
			fmt.Println()

			if !bytes.Equal(p, cp) {
				fmt.Println(color.Red("Passwords not matching"))
				os.Exit(1)
			}

			pwd = p
		}

		res, err := api.CreateUser(ctx, &authv1beta1.CreateUserRequest{
			User: &authv1beta1.User{
				Username: uid,
			},
			Password: strings.TrimSuffix(string(pwd), "\n"),
		})
		if err != nil {
			err := err.(twirp.Error)
			jww.ERROR.Fatal(err.Msg())
		}

		user := res.GetUser()
		fmt.Printf("Created user %s", color.Blue(user.GetUsername()))
		fmt.Println()
	},
}

func init() {
	user.Flags().Bool("password-stdin", false, "Take the password from stdin")
}
