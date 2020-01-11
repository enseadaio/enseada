// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package config

import (
	"encoding/json"
	"io/ioutil"
	"os"
	"time"

	"github.com/hashicorp/hcl"
	"github.com/hashicorp/hcl/hcl/printer"
)

type Config struct {
	Profiles map[string]Profile `hcl:"profile" json:"profile"`
}

type Profile struct {
	URL          string    `hcl:"url,omitempty" json:"url,omitempty"`
	RefreshToken string    `hcl:"token,omitempty" json:"token,omitempty"`
	Expiry       time.Time `hcl:"-" json:"-"`
}

func Read(filename string) (*Config, error) {
	c := new(Config)
	f, err := ioutil.ReadFile(filename)
	if err != nil {
		return nil, err
	}

	if err := hcl.Unmarshal(f, c); err != nil {
		return nil, err
	}

	for n, p := range c.Profiles {
		if p.Expiry.IsZero() {
			p.Expiry = p.Expiry.Add(time.Second) // make it non-zero to force token refresh
			c.Profiles[n] = p
		}
	}

	return c, nil
}

func Write(filename string, c *Config) error {
	f, err := os.OpenFile(filename, os.O_WRONLY, os.ModePerm)
	if err != nil {
		return err
	}
	defer f.Close()

	b, err := json.Marshal(c)
	if err != nil {
		return err
	}
	s := string(b)
	ast, err := hcl.Parse(s)
	if err != nil {
		return err
	}
	err = printer.Fprint(f, ast.Node)
	if err != nil {
		return err
	}
	return nil
}
