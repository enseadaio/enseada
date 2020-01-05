// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package stackdriver

import (
	"context"
	"fmt"

	"cloud.google.com/go/errorreporting"
	"github.com/enseadaio/enseada/pkg/errare"
	"google.golang.org/api/option"
)

type Handler struct {
	client *errorreporting.Client
}

func NewHandler(ctx context.Context, projectId string, conf errorreporting.Config, opts ...option.ClientOption) (*Handler, error) {
	client, err := errorreporting.NewClient(ctx, projectId, conf, opts...)
	if err != nil {
		return nil, err
	}
	return &Handler{client: client}, nil
}

func (h *Handler) HandleError(err error) {
	h.client.Report(errorreporting.Entry{
		Error: err,
	})
}

func (h *Handler) HandlePanic(v interface{}) {
	err, ok := v.(error)
	if !ok {
		err = fmt.Errorf("%v", v)
	}
	h.HandleError(err)
}

func (h *Handler) HandlePanicWithContext(ctx context.Context, v interface{}) {
	h.HandlePanic(v)
}

func (h *Handler) SetCurrentUser(id string, extras errare.Extras) {
}

func (h *Handler) Close() error {
	return h.client.Close()
}
