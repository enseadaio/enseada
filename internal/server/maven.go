// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package server

import (
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/labstack/echo"
	"io/ioutil"
	"net/http"
	"strings"
)

func mountMaven(e *echo.Echo, mvn *maven.Maven) {
	m := e.Group("/maven2")

	m.GET("/*", getMaven(mvn))
	m.PUT("/*", storeMaven(mvn))

}

func getMaven(mvn *maven.Maven) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		path := strings.TrimPrefix(c.Request().RequestURI, "/")
		c.Logger().Infof("Loading file from %s", path)
		file, err := mvn.GetFile(ctx, path)
		if err != nil {
			return err
		}

		if file != nil {
			c.Logger().Info("File found")
			return c.Blob(http.StatusOK, "application/octet-stream", file.Content)
		}

		c.Logger().Warnf("No file found at %s", path)
		return c.NoContent(http.StatusNotFound)
	}
}

func storeMaven(mvn *maven.Maven) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		path := strings.TrimPrefix(c.Request().RequestURI, "/maven2/")
		c.Logger().Infof("attempting storing Maven artifact at %s", path)
		body, err := ioutil.ReadAll(c.Request().Body)
		if err != nil {
			return err
		}

		file, err := mvn.PutRepoFile(ctx, path, body)
		if err != nil {
			c.Logger().Error(err)
			if err == maven.ErrorRepoNotFound {
				return c.NoContent(http.StatusNotFound)
			}
			return err
		}

		c.Logger().Info("stored Maven artifact %s at %s", file.Filename, path)
		return c.NoContent(http.StatusCreated)
	}
}
