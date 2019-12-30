// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package server

import (
	"net/http"
	"strings"

	"github.com/enseadaio/enseada/internal/maven"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	goauth "golang.org/x/oauth2"
)

type RouteParams struct {
	Echo          *echo.Echo
	Mvn           *maven.Maven
	OAuthProvider fosite.OAuth2Provider
	OAuthClient   *goauth.Config
	PublicHost    string
	SecretKeyBase []byte
}

func MountRoutes(p RouteParams) {
	e := p.Echo
	mvn := p.Mvn
	oauth := p.OAuthProvider
	oc := p.OAuthClient
	//host := p.PublicHost
	secret := p.SecretKeyBase

	mountRoot(e)
	mountMaven(e, mvn)
	mountHealthCheck(e)
	mountUI(e, oc, secret)
	mountOauth(e, oauth)
}

func mountRoot(e *echo.Echo) {
	e.GET("/", func(c echo.Context) error {
		acc := c.Request().Header.Get("accept")

		if strings.Contains(acc, "html") {
			return c.Redirect(http.StatusMovedPermanently, "/ui")
		}

		return c.JSON(http.StatusNotFound, echo.Map{
			"error":   "not_found",
			"message": "NotFound",
		})
	})
}
