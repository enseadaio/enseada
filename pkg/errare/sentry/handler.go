// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package sentry

import (
	"context"
	"errors"
	"time"

	"github.com/enseadaio/enseada/pkg/errare"
	"github.com/getsentry/sentry-go"
)

type Handler struct {
	hub     *sentry.Hub
	client  *sentry.Client
	timeout time.Duration
}

func NewHandler(opts sentry.ClientOptions) (*Handler, error) {
	client, err := sentry.NewClient(opts)
	if err != nil {
		return nil, err
	}
	hub := sentry.NewHub(client, sentry.NewScope())

	return &Handler{
		hub:     hub,
		client:  client,
		timeout: 5 * time.Second,
	}, nil
}

func (s *Handler) HandleError(err error) {
	s.hub.CaptureException(err)
}

func (s *Handler) HandlePanic(v interface{}) {
	s.hub.Recover(v)
}

func (s *Handler) HandlePanicWithContext(ctx context.Context, v interface{}) {
	s.hub.RecoverWithContext(ctx, v)
}

func (s *Handler) SetCurrentUser(id string, extras errare.Extras) {
	s.hub.ConfigureScope(func(scope *sentry.Scope) {
		scope.SetUser(sentry.User{
			Email:     extras.GetOr("email", "").(string),
			ID:        id,
			IPAddress: extras.GetOr("ip", "").(string),
			Username:  extras.GetOr("username", "").(string),
		})
	})
}

func (s *Handler) Close() error {
	if !s.hub.Flush(s.timeout) {
		return errors.New("timeout exceeded")
	}
	return nil
}
