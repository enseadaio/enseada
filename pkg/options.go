// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package enseada

import "github.com/labstack/gommon/log"

type ServerOptions struct {
	level                    log.Lvl
	publicHost               string
	defaultOauthClientSecret string
	rootUserPassword         string
	secretKeyBase            string
}

type ServerOption func(opts *ServerOptions)

func ServerLogLevel(level log.Lvl) ServerOption {
	return func(opts *ServerOptions) {
		opts.level = level
	}
}

func ServerDefaultOAuthClientSecret(secret string) ServerOption {
	return func(opts *ServerOptions) {
		opts.defaultOauthClientSecret = secret
	}
}

func ServerPublicHost(host string) ServerOption {
	return func(opts *ServerOptions) {
		opts.publicHost = host
	}
}

func ServerSecretKeyBase(secret string) ServerOption {
	return func(opts *ServerOptions) {
		opts.secretKeyBase = secret
	}
}
