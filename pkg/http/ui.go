// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"strings"

	"github.com/labstack/echo/middleware"

	rice "github.com/GeertJohan/go.rice"
	"github.com/enseadaio/enseada/internal/auth"
	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/random"
	goauth "golang.org/x/oauth2"
)

func mountUI(e *echo.Echo, oc *goauth.Config, sm echo.MiddlewareFunc) {
	e.GET("/", root)

	staticHandler := http.FileServer(rice.MustFindBox("../../web/static").HTTPBox())
	e.GET("/static/*", echo.WrapHandler(http.StripPrefix("/static/", staticHandler)))

	assetHandler := http.FileServer(rice.MustFindBox("../../web/assets").HTTPBox())
	e.GET("/assets/*", echo.WrapHandler(http.StripPrefix("/assets/", assetHandler)))

	u := e.Group("/ui")
	u.Use(sm)
	u.Use(middleware.CSRF())
	u.GET("", home(oc))
	u.GET("/callback", callback(oc))
	u.GET("/error", errorPage(oc))
}

func home(oc *goauth.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		return renderPage(c, http.StatusOK, "index", oc, echo.Map{})
	}
}

func callback(oc *goauth.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		code := c.QueryParam("code")
		ctx := c.Request().Context()
		c.Logger().Info(code)
		t, err := oc.Exchange(ctx, code)
		if err != nil {
			return err
		}

		cl := &http.Client{}
		v := url.Values{}
		v.Set("token", t.AccessToken)
		v.Set("token_type_hint", "access_token")
		req, err := http.NewRequest("POST", oc.Endpoint.TokenURL+"/introspect", strings.NewReader(v.Encode()))
		if err != nil {
			return err
		}

		req.SetBasicAuth(oc.ClientID, oc.ClientSecret)
		req.Header.Set("content-type", "application/x-www-form-urlencoded")
		req.Header.Add("Accept-Encoding", "identity")
		res, err := cl.Do(req)
		if err != nil {
			return err
		}

		defer res.Body.Close()

		b, err := ioutil.ReadAll(res.Body)
		if err != nil {
			return err
		}

		var body map[string]interface{}
		err = json.Unmarshal(b, &body)
		if err != nil {
			return err
		}

		s := session.Default(c)
		if body["active"] == true {
			s.Set("access_token", t.AccessToken)
			s.Set("refresh_token", t.RefreshToken)
			s.Set("current_user_id", body["sub"].(string))
			s.Set("current_user_name", body["username"].(string))
			err = s.Save()
			if err != nil {
				return err
			}
		} else {
			s.Clear()
			s.AddFlash(fmt.Sprintf("%s. %s", body["error_description"], body["error_hint"]), "errors")
		}

		return c.Redirect(http.StatusTemporaryRedirect, "/ui")
	}
}

func root(c echo.Context) error {
	acc := c.Request().Header.Get("accept")

	if strings.Contains(acc, "html") {
		return c.Redirect(http.StatusMovedPermanently, "/ui")
	}

	return echo.ErrNotFound
}

func errorPage(oc *goauth.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		if err := http2Push(c); err != nil {
			return err
		}

		params := echo.Map{
			"Title": "Error",
		}
		s := session.Default(c)
		errs := s.Flashes("errors")
		he := s.Flashes("HTTPError")
		if len(errs) == 0 && len(he) == 0 {
			return c.Redirect(http.StatusTemporaryRedirect, "/ui")
		}

		params["Errors"] = errs
		if len(he) > 0 {
			params["HTTPError"] = he[0]
		}
		addCurrentUser(s, params)
		params["LoginURL"] = oc.AuthCodeURL(random.String(32))
		if err := s.Save(); err != nil {
			return err
		}
		return c.Render(http.StatusBadRequest, "error", params)
	}
}

func renderPage(c echo.Context, sc int, name string, oc *goauth.Config, params echo.Map) error {
	if err := http2Push(c); err != nil {
		return err
	}

	s := session.Default(c)
	addFlashes(s, params)
	addCurrentUser(s, params)
	params["LoginURL"] = oc.AuthCodeURL(random.String(32))
	if err := s.Save(); err != nil {
		return err
	}
	return c.Render(sc, name, params)
}

func http2Push(c echo.Context) error {
	pusher, ok := c.Response().Writer.(http.Pusher)
	if ok {
		if err := pusher.Push("/static/main.css", nil); err != nil {
			return err
		}
		if err := pusher.Push("/static/runtime.js", nil); err != nil {
			return err
		}
		if err := pusher.Push("/static/app.js", nil); err != nil {
			return err
		}
	}
	return nil
}

func addCurrentUser(s session.Session, params echo.Map) {
	i := s.Get("current_user_id")
	u := s.Get("current_user_name")
	if i != nil && u != nil {
		params["CurrentUser"] = auth.User{
			Username: i.(string),
		}
	}
}

func addFlashes(s session.Session, params echo.Map) {
	errs := s.Flashes("errors")
	params["Errors"] = errs
	he := s.Flashes("HTTPError")
	if len(he) > 0 {
		params["HTTPError"] = he[0]
	}
}
