// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package config

import (
	"context"
	"sync"

	enseada "github.com/enseadaio/enseada/pkg"
	jww "github.com/spf13/jwalterweatherman"
	"github.com/spf13/viper"
	goauth "golang.org/x/oauth2"
)

func Client(ctx context.Context) enseada.Client {
	url := viper.GetString("url")
	c := goauth.NewClient(ctx, tokens(ctx))
	return enseada.NewClient(url, c)
}

func OAuthConfig(url string) *goauth.Config {
	return &goauth.Config{
		ClientID: "enseada-cli",
		Endpoint: goauth.Endpoint{
			AuthURL:  url + "/oauth/authorize",
			TokenURL: url + "/oauth/token",
		},
		RedirectURL: "http://localhost:19999/",
		Scopes:      []string{"*"},
	}
}

func tokens(ctx context.Context) goauth.TokenSource {
	if t := viper.Get("tokens"); t != nil {
		tkn := t.(*goauth.Token)
		if tkn.RefreshToken != "" && !tkn.Expiry.IsZero() {
			url := viper.GetString("url")
			oc := OAuthConfig(url)
			return &refreshableTokenSource{
				t:   tkn,
				new: oc.TokenSource(ctx, tkn),
			}
		}
	}
	jww.ERROR.Fatal("You are not logged in. Please run `enseada login`.")
	return nil
}

type refreshableTokenSource struct {
	new goauth.TokenSource
	mu  sync.Mutex // guards t
	t   *goauth.Token
}

func (r *refreshableTokenSource) Token() (*goauth.Token, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.t.Valid() {
		return r.t, nil
	}
	t, err := r.new.Token()
	if err != nil {
		return nil, err
	}
	r.t = t

	c := viper.Get("_config").(*Config)
	p := c.Profiles[viper.GetString("profile")]
	p.RefreshToken = t.RefreshToken
	p.Expiry = t.Expiry
	c.Profiles[viper.GetString("profile")] = p
	if err := Write(viper.GetString("config"), c); err != nil {
		return nil, err
	}

	return t, nil
}
