// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package log

import (
	"context"

	"github.com/enseadaio/enseada/pkg/errare"
	"github.com/enseadaio/enseada/pkg/log"
)

type Handler struct {
	logger      log.Logger
	exitOnPanic bool
}

func NewHandler(logger log.Logger, exitOnPanic bool) *Handler {
	return &Handler{logger: logger, exitOnPanic: exitOnPanic}
}

func (h *Handler) HandleError(err error) {
	h.logger.Error(err)
}

func (h *Handler) HandlePanic(v interface{}) {
	if h.exitOnPanic {
		h.logger.Fatalf("panic: %v", v)
	}

	h.logger.Errorf("panic: %v")
}

func (h *Handler) HandlePanicWithContext(ctx context.Context, v interface{}) {
	h.HandlePanic(v)
}

func (h *Handler) SetCurrentUser(id string, extras errare.Extras) {
}

func (h *Handler) Close() error {
	return nil
}
