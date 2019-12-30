// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"runtime"
	"sync"
)

type CallbackFunc func(string)

type CasbinWatcher struct {
	logger   echo.Logger
	data     *kivik.Client
	callback CallbackFunc
	ch       *kivik.Changes
	once     sync.Once
}

func NewCasbinWatcher(data *kivik.Client, logger echo.Logger) *CasbinWatcher {
	w := &CasbinWatcher{
		logger: logger,
		data:   data,
	}

	return w
}

// SetUpdateCallback sets the callback function that the watcher will call
// when the policy in DB has been changed by other instances.
// A classic callback is Enforcer.LoadPolicy().
func (w *CasbinWatcher) SetUpdateCallback(cb func(string)) error {
	w.callback = cb
	return nil
}

// Update calls the update callback of other instances to synchronize their policy.
// It is usually called after changing the policy in DB, like Enforcer.SavePolicy(),
// Enforcer.AddPolicy(), Enforcer.RemovePolicy(), etc.
func (w *CasbinWatcher) Update() error {
	// noop because Couch provides the update for us via the Changes feed
	return nil
}

// Close stops and releases the watcher, the callback function will not be called any more.
func (w *CasbinWatcher) Close() {
	finalizer(w)
}

func (w *CasbinWatcher) Start(ctx context.Context) error {
	w.logger.Info("started acl watcher")
	db := w.data.DB(ctx, couch.AclDB)
	ch, err := db.Changes(ctx, kivik.Options{
		"feed": "continuous",
	})
	if err != nil {
		return err
	}

	w.ch = ch

	runtime.SetFinalizer(w, finalizer)

	go func() {
		select {
		case <-ctx.Done():
			w.logger.Error(ctx.Err())
			return
		default:
			for ch.Next() {
				w.logger.Debugf("received change from feed. id: %s", ch.ID())
				w.callback(ch.ID())
			}

			w.logger.Warn("no more changes. Stopping...")
		}
	}()

	return nil
}

func finalizer(w *CasbinWatcher) {
	w.once.Do(func() {
		if err := w.ch.Close(); err != nil {
			w.logger.Error(err)
		}
	})
}
