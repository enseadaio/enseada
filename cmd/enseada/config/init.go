// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package config

import (
	"os"
	"path/filepath"

	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

func InitConfigFile() {
	cf := viper.GetString("config")
	if filepath.Ext(cf) != ".hcl" {
		jww.ERROR.Fatal("Config file must be an HCL file")
	}

	_, err := os.Stat(cf)
	if err == nil {
		return
	}

	if !os.IsNotExist(err) {
		jww.ERROR.Fatal(err)
	}

	if err := os.MkdirAll(filepath.Dir(cf), 0755); err != nil {
		jww.ERROR.Fatal(err)
	}

	emptyFile, err := os.Create(cf)
	if err != nil {
		jww.ERROR.Fatal(err)
	}

	if err := emptyFile.Close(); err != nil {
		jww.ERROR.Fatal(err)
	}
}

func ReadConfigFile() {
	cf := viper.GetString("config")
	c, err := Read(cf)
	if err != nil {
		jww.ERROR.Fatal(err)
	}

	viper.Set("_config", c)

	pn := viper.GetString("profile")
	p := c.Profiles[pn]

	viper.Set("url", p.URL)
	viper.Set("tokens", &goauth.Token{
		RefreshToken: p.RefreshToken,
		Expiry:       p.Expiry,
	})
}
