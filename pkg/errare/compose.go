// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package errare

import (
	"context"

	"go.uber.org/multierr"
)

func Compose(handlers ...Handler) Handler {
	return &compositeHandler{hh: handlers}
}

type compositeHandler struct {
	hh []Handler
}

func (c *compositeHandler) Close() error {
	var err error
	for _, h := range c.hh {
		e := h.Close()
		err = multierr.Append(err, e)
	}
	return err
}

func (c *compositeHandler) HandleError(err error) {
	for _, h := range c.hh {
		h.HandleError(err)
	}
}

func (c *compositeHandler) HandlePanic(v interface{}) {
	for _, h := range c.hh {
		h.HandlePanic(v)
	}
}

func (c *compositeHandler) HandlePanicWithContext(ctx context.Context, v interface{}) {
	for _, h := range c.hh {
		h.HandlePanicWithContext(ctx, v)
	}
}

func (c *compositeHandler) SetCurrentUser(id string, extras Extras) {
	for _, h := range c.hh {
		h.SetCurrentUser(id, extras)
	}
}
