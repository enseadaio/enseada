package enseada

import "github.com/labstack/gommon/log"

type ServerOptions struct {
	level log.Lvl
}

type ServerOption func(opts *ServerOptions) error

func ServerLogLevel(level log.Lvl) ServerOption {
	return func(opts *ServerOptions) error {
		opts.level = level
		return nil
	}
}
