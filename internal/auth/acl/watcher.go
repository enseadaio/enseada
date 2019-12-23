package acl

import (
	"context"
	"github.com/casbin/casbin/v2/persist"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"runtime"
	"sync"
)

type CallbackFunc func(string)

type Watcher struct {
	logger   echo.Logger
	data     *kivik.Client
	callback CallbackFunc
	ch       *kivik.Changes
	once     sync.Once
}

func NewWatcher(ctx context.Context, client *kivik.Client, logger echo.Logger) persist.Watcher {
	w := &Watcher{
		logger: logger,
		data:   client,
	}

	go func() {
		select {
		case <-ctx.Done():
			logger.Error(ctx.Err())
			return
		default:
			err := w.Start(ctx)
			if err != nil {
				logger.Error(err)
			}
		}
	}()

	return w
}

// SetUpdateCallback sets the callback function that the watcher will call
// when the policy in DB has been changed by other instances.
// A classic callback is Enforcer.LoadPolicy().
func (w *Watcher) SetUpdateCallback(cb func(string)) error {
	w.callback = cb
	return nil
}

// Update calls the update callback of other instances to synchronize their policy.
// It is usually called after changing the policy in DB, like Enforcer.SavePolicy(),
// Enforcer.AddPolicy(), Enforcer.RemovePolicy(), etc.
func (w *Watcher) Update() error {
	// noop because Couch provides the update for us via the Changes feed
	return nil
}

// Close stops and releases the watcher, the callback function will not be called any more.
func (w *Watcher) Close() {
	finalizer(w)
}

func (w *Watcher) Start(ctx context.Context) error {
	w.logger.Info("started acl watcher")
	db := w.data.DB(ctx, couch.AclDB)
	ch, err := db.Changes(ctx, kivik.Options{
		"feed": "continuous",
	})
	if err != nil {
		return err
	}

	runtime.SetFinalizer(w, finalizer)

	w.ch = ch
	for ch.Next() {
		w.logger.Debugf("received change from feed. id: %s", ch.ID())
		w.callback(ch.ID())
	}

	w.logger.Warn("no more changes. Stopping...")
	return nil
}

func finalizer(w *Watcher) {
	w.once.Do(func() {
		if err := w.ch.Close(); err != nil {
			w.logger.Error(err)
		}
	})
}
