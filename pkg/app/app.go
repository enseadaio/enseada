// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package app

import (
	"context"
	"sync"
	"time"

	"github.com/labstack/gommon/log"
)

type Module interface {
	Start(ctx context.Context) error
	Stop(ctx context.Context) error
}

type EventHandlersMap map[LifecycleEvent]OnLifecycleEventFunc

type LifecycleAware interface {
	EventHandlers() EventHandlersMap
}

type App struct {
	modules    []Module
	lcHandlers map[LifecycleEvent][]OnLifecycleEventFunc
	onError    func(err error)
	onPanic    func(v interface{})
}

func New(options ...Option) *App {
	opts := &Options{
		modules: []Module{},
		onError: func(err error) {
			log.Error(err)
		},
		onPanic: func(v interface{}) {
			panic(v)
		},
	}

	for _, o := range options {
		o(opts)
	}

	a := &App{
		lcHandlers: make(map[LifecycleEvent][]OnLifecycleEventFunc),
	}

	a.modules = opts.modules
	a.onError = opts.onError
	a.onPanic = opts.onPanic

	for _, m := range a.modules {
		em, ok := m.(LifecycleAware)
		if ok {
			for e, h := range em.EventHandlers() {
				a.lcHandlers[e] = append(a.lcHandlers[e], h)
			}
		}

	}
	return a
}

func (a *App) Start(ctx context.Context) error {
	defer recoverPanic(a)

	errs := make(chan error)
	a.emitEvent(ctx, BeforeApplicationStartEvent)
	for _, m := range a.modules {
		go func(m Module) {
			defer recoverPanic(a)

			if err := m.Start(ctx); err != nil {
				errs <- err
			}
		}(m)
	}

	startctx, cancel := context.WithTimeout(ctx, 1*time.Second)
	defer cancel()

	select {
	case err := <-errs:
		a.onError(err)
		if err := a.Stop(ctx); err != nil {
			return err
		}
		return err
	case <-startctx.Done():
		a.emitEvent(ctx, AfterApplicationStartEvent)
		return nil
	}
}

func (a *App) Stop(ctx context.Context) error {
	defer recoverPanic(a)

	a.emitEvent(ctx, BeforeApplicationStopEvent)
	for _, m := range a.modules {
		if err := m.Stop(ctx); err != nil {
			a.onError(err)
			return err
		}
	}

	a.emitEvent(ctx, AfterApplicationStopEvent)
	return nil
}

func (a *App) emitEvent(ctx context.Context, e LifecycleEvent) {
	var w sync.WaitGroup
	for _, eh := range a.lcHandlers[e] {
		w.Add(1)
		go func(eh OnLifecycleEventFunc) {
			defer w.Done()
			eh(ctx, e)
		}(eh)
	}

	w.Wait()
}

func recoverPanic(a *App) {
	if r := recover(); r != nil {
		a.onPanic(r)
	}
}
