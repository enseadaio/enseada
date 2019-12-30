// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"context"
	rice "github.com/GeertJohan/go.rice"
	"github.com/casbin/casbin/v2"
	"github.com/casbin/casbin/v2/model"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
)

type CasbinLogger echo.Logger

func casbinLog(lvl log.Lvl) CasbinLogger {
	casbinLogger := log.New("casbin")
	casbinLogger.SetLevel(lvl)
	return casbinLogger
}

func casbinModel() (model.Model, error) {
	box := rice.MustFindBox("../../../conf/")
	return model.NewModelFromString(box.MustString("casbin_model.conf"))
}

func casbinAdapter(data *kivik.Client, logger CasbinLogger) (*auth.CasbinAdapter, error) {
	return auth.NewCasbinAdapter(data, logger)
}

func casbinWatcher(data *kivik.Client, logger CasbinLogger) *auth.CasbinWatcher {
	w := auth.NewCasbinWatcher(data, logger)
	return w
}

func casbinEnforcer(ctx context.Context, model model.Model, adapter *auth.CasbinAdapter, watcher *auth.CasbinWatcher) (*casbin.Enforcer, error) {
	e, err := casbin.NewEnforcer(model, adapter)
	if err != nil {
		return nil, err
	}

	err = e.SetWatcher(watcher)
	if err != nil {
		return nil, err
	}
	e.EnableLog(false)
	e.EnableAutoSave(true)

	err = watcher.Start(ctx)
	return e, err
}
