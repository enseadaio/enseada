// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package airbrake

import (
	"context"

	"github.com/airbrake/gobrake/v4"
	"github.com/enseadaio/enseada/pkg/errare"
)

type Handler struct {
	notifier *gobrake.Notifier
}

func NewHandler(opts *gobrake.NotifierOptions) *Handler {
	notifier := gobrake.NewNotifierWithOptions(opts)
	return &Handler{notifier: notifier}
}

func (h *Handler) HandleError(err error) {
	h.notifier.Notify(err, nil)
}

func (h *Handler) HandlePanic(v interface{}) {
	notice := h.notifier.Notice(v, nil, 2)
	notice.Context["severity"] = "critical"
	h.notifier.SendNoticeAsync(notice)
}

func (h *Handler) HandlePanicWithContext(ctx context.Context, v interface{}) {
	h.HandlePanic(v)
}

func (h *Handler) SetCurrentUser(id string, extras errare.Extras) {
}

func (h *Handler) Close() error {
	h.notifier.Flush()
	return h.notifier.Close()
}
