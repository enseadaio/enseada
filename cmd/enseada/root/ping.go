// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package root

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"os"
	"time"

	"github.com/spf13/cobra"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"
)

var pingCmd = &cobra.Command{
	Use:   "ping",
	Short: "Ping the server and get its status",
	Run: func(cmd *cobra.Command, args []string) {
		tout := 10 * time.Second

		c := &http.Client{
			Timeout: tout,
		}

		base, err := url.ParseRequestURI(viper.GetString("url"))
		if err != nil {
			jww.ERROR.Fatal(err)
		}

		h, err := base.Parse("/health")
		if err != nil {
			jww.ERROR.Fatal(err)
		}

		res, err := c.Get(h.String())
		if err != nil {
			jww.ERROR.Fatal(err)
		}

		defer res.Body.Close()
		b, err := ioutil.ReadAll(res.Body)
		bb := new(bytes.Buffer)
		if err := json.Indent(bb, b, "", "  "); err != nil {
			jww.ERROR.Fatal(err)
		}

		fmt.Println(bb.String())
		if res.StatusCode > 399 {
			os.Exit(1)
		}
	},
}
