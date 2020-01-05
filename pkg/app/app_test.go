// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package app

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

type TestModule struct {
	t                 *testing.T
	BeforeStartCalled bool
	StartCalled       bool
	AfterStartCalled  bool
	BeforeStopCalled  bool
	StopCalled        bool
	AfterStopCalled   bool
}

func (m *TestModule) EventHandlers() EventHandlersMap {
	return EventHandlersMap{
		BeforeApplicationStartEvent: func(ctx context.Context, event LifecycleEvent) error {
			assert.Equal(m.t, BeforeApplicationStartEvent, event)
			m.BeforeStartCalled = true
			return nil
		},
		AfterApplicationStartEvent: func(ctx context.Context, event LifecycleEvent) error {
			assert.Equal(m.t, AfterApplicationStartEvent, event)
			m.AfterStartCalled = true
			return nil
		},
		BeforeApplicationStopEvent: func(ctx context.Context, event LifecycleEvent) error {
			assert.Equal(m.t, BeforeApplicationStopEvent, event)
			m.BeforeStopCalled = true
			return nil
		},
		AfterApplicationStopEvent: func(ctx context.Context, event LifecycleEvent) error {
			assert.Equal(m.t, AfterApplicationStopEvent, event)
			m.AfterStopCalled = true
			return nil
		},
	}
}

func (m *TestModule) Start(ctx context.Context) error {
	m.StartCalled = true
	return nil
}

func (m *TestModule) Stop(ctx context.Context) error {
	m.StopCalled = true
	return nil
}

func (m *TestModule) End() {
	assert.True(m.t, m.BeforeStartCalled)
	assert.True(m.t, m.StartCalled)
	assert.True(m.t, m.AfterStartCalled)
	assert.True(m.t, m.BeforeStopCalled)
	assert.True(m.t, m.StopCalled)
	assert.True(m.t, m.AfterStopCalled)
}

func TestNewApp(t *testing.T) {
	ctx, cancel := context.WithCancel(context.TODO())
	defer cancel()

	m := &TestModule{t: t}
	a := New(
		Modules(m),
		OnError(func(err error) {
			assert.Fail(t, "received error:", err.Error())
		}),
		OnPanic(func(err error) {
			assert.Fail(t, "received panic:", err.Error())
		}),
	)

	if err := a.Start(ctx); err != nil {
		assert.NoError(t, err)
	}

	if err := a.Stop(ctx); err != nil {
		assert.NoError(t, err)
	}

	m.End()
}
