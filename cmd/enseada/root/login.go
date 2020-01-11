// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package root

import (
	"context"
	"crypto/rand"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"time"

	"github.com/enseadaio/enseada/cmd/enseada/config"
	"github.com/labstack/gommon/color"
	"github.com/pkg/browser"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"

	"github.com/spf13/cobra"
)

const response = `
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<title>Enseada CLI</title>
</head>
<body>
You can now close this window.
</body>
</html>
`

var loginCmd = &cobra.Command{
	Use:   "login [url]",
	Short: "Login with an Enseada server",
	Args: func(cmd *cobra.Command, args []string) error {
		if len(args) > 1 {
			return fmt.Errorf("accepts 1 arg, received %d", len(args))
		}

		u := viper.GetString("url")
		if len(args) > 0 {
			u = args[0]
		}

		if u == "" {
			return fmt.Errorf("no URL provided. Pass it as the first argument to this command")
		}

		_, err := url.ParseRequestURI(u)
		if err != nil {
			return err
		}

		viper.Set("url", u)
		return nil
	},
	Run: func(cmd *cobra.Command, args []string) {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
		defer cancel()

		u := viper.GetString("url")
		tkn := new(goauth.Token)
		oc := config.OAuthConfig(u)

		srv := &http.Server{
			Addr: ":19999",
		}

		srv.Handler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			q := r.URL.Query()

			if err := q.Get("error"); err != "" {
				desc := q.Get("error_description")
				hint := q.Get("error_hint")
				jww.ERROR.Println(err)
				jww.ERROR.Println(desc)
				jww.ERROR.Println(hint)
				if err := srv.Shutdown(ctx); err != nil {
					jww.DEBUG.Println(err)
				}
				os.Exit(1)
			}

			c := q.Get("code")
			t, err := oc.Exchange(ctx, c)
			if err != nil {
				jww.ERROR.Fatal(err)
			}

			tkn = t

			if _, err := w.Write([]byte(response)); err != nil {
				jww.ERROR.Fatal(err)
			}

			if err := srv.Shutdown(ctx); err != nil {
				jww.DEBUG.Println(err)
			}
		})

		b := make([]byte, 32)
		if _, err := rand.Read(b); err != nil {
			jww.ERROR.Fatal(err)
		}
		state := fmt.Sprintf("%x", b)
		acu := oc.AuthCodeURL(state)
		fmt.Println("You'll be redirected to the server for authentication.")
		fmt.Println("If a browser windows does not open automatically, copy this URL.")
		fmt.Println(color.Cyan(acu))

		if err := browser.OpenURL(acu); err != nil {
			jww.ERROR.Fatal(err)
		}

		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			jww.ERROR.Fatal(err)
		}

		if tkn == nil {
			jww.ERROR.Fatal("No tokens after callback, something's wrong...")
		}

		viper.Set("tokens", tkn)

		c := viper.Get("_config").(*config.Config)
		if len(c.Profiles) == 0 {
			c.Profiles = make(map[string]config.Profile)
		}

		p := c.Profiles[viper.GetString("profile")]
		p.URL = u
		p.RefreshToken = tkn.RefreshToken
		c.Profiles[viper.GetString("profile")] = p
		if err := config.Write(viper.GetString("config"), c); err != nil {
			jww.ERROR.Fatal(err)
		}

		fmt.Println(color.Blue("Authentication successful"))
	},
}
