// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"io/ioutil"
	"net/http"
	"strings"

	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/maven"
	mavenv1beta1api "github.com/enseadaio/enseada/internal/maven/v1beta1"
	"github.com/enseadaio/enseada/internal/middleware"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/ory/fosite"

	"github.com/labstack/echo"
)

func mountRoutes(e *echo.Echo, m *maven.Maven, s *auth.Store, op fosite.OAuth2Provider) {
	g := e.Group("/maven2")

	g.GET("/*", getMaven(m))
	g.PUT("/*", storeMaven(m))

	mvnsvc := mavenv1beta1api.Service{Maven: m}
	mvnHandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	h := echo.WrapHandler(middleware.WithAuthorizationHeader(mvnHandler, m.Logger, s, op))
	e.Any(mvnHandler.PathPrefix()+"*", h)
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
