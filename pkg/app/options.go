// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package app

type Options struct {
	modules []Module
	onError func(err error)
	onPanic func(v interface{})
}

type Option func(*Options)

func Modules(ms ...Module) Option {
	return func(opts *Options) {
		opts.modules = ms
	}
}

func OnError(f func(err error)) Option {
	return func(opts *Options) {
		opts.onError = f
	}
}

func OnPanic(f func(v interface{})) Option {
	return func(opts *Options) {
		opts.onPanic = f
	}
}
