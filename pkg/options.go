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
