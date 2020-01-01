// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"net/http"

	"github.com/enseadaio/enseada/internal/utils"
	"github.com/labstack/echo"
	"github.com/labstack/echo/middleware"
	"github.com/labstack/gommon/log"
)

func createEchoServer(lvl log.Lvl) *echo.Echo {
	e := echo.New()

	e.Logger.SetLevel(lvl)
	e.HideBanner = true
	e.HTTPErrorHandler = func(err error, c echo.Context) {
		e := c.JSON(http.StatusInternalServerError, utils.HTTPError(http.StatusInternalServerError, err.Error()))
		if e != nil {
			c.Logger().Error(e)
		}
	}

	e.Renderer = NewGoViewRenderer()

	e.Use(middleware.Recover())
	e.Use(middleware.CORS())
	e.Use(middleware.RequestID())
	e.Use(middleware.Logger())
	e.Use(middleware.GzipWithConfig(middleware.GzipConfig{
		Level: 5,
	}))
	e.Pre(middleware.RemoveTrailingSlashWithConfig(
		middleware.TrailingSlashConfig{
			RedirectCode: http.StatusMovedPermanently,
		}))

	return e
}
