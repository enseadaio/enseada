// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package server

import (
	"encoding/json"
	"io/ioutil"
	"net/http"
	"net/url"
	"strings"
	"time"

	rice "github.com/GeertJohan/go.rice"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/go-session/cookie"
	echosession "github.com/go-session/echo-session"
	"github.com/go-session/session"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/random"
	"golang.org/x/oauth2"
)

func mountUI(e *echo.Echo, oc oauth2.Config, skb []byte) {
	staticHandler := http.FileServer(rice.MustFindBox("../../web/static").HTTPBox())
	e.GET("/static/*", echo.WrapHandler(http.StripPrefix("/static/", staticHandler)))

	assetHandler := http.FileServer(rice.MustFindBox("../../web/assets").HTTPBox())
	e.GET("/assets/*", echo.WrapHandler(http.StripPrefix("/assets/", assetHandler)))

	u := e.Group("/ui")

	exp := (time.Hour * 720).Seconds()
	store := cookie.NewCookieStore(
		cookie.SetCookieName("enseada-session"),
		cookie.SetHashKey(skb),
	)
	u.Use(echosession.New(
		session.SetCookieName("enseada-session-id"),
		session.SetExpired(int64(exp)),
		session.SetStore(store),
	))

	u.GET("", home(oc))
	u.GET("/profile", profile(oc))
	u.GET("/repositories", repos(oc))
	u.GET("/callback", callback(oc))
}

func home(oc oauth2.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		return renderPage(c, "index", oc, echo.Map{})
	}
}

func repos(oc oauth2.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		return renderPage(c, "repos", oc, echo.Map{})
	}
}

func profile(oc oauth2.Config) echo.HandlerFunc {
	return func(c echo.Context) error {
		s := echosession.FromContext(c)
		_, ok := s.Get("current_user_id")
		if !ok {
			return c.Redirect(http.StatusSeeOther, oc.AuthCodeURL(random.String(32)))
		}
		return renderPage(c, "profile", oc, echo.Map{})
	}
}

func callback(oc oauth2.Config) echo.HandlerFunc {
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

		store := echosession.FromContext(c)
		if body["active"] == true {
			store.Context()
			store.Set("access_token", t.AccessToken)
			store.Set("refresh_token", t.RefreshToken)
			store.Set("current_user_id", body["sub"].(string))
			store.Set("current_user_name", body["username"].(string))
			err = store.Save()
			if err != nil {
				return err
			}
		} else {
			err := store.Flush()
			if err != nil {
				return err
			}
			err = echosession.Destroy(c)
			if err != nil {
				return err
			}
		}

		return c.Redirect(http.StatusTemporaryRedirect, "/ui")
	}
}

func renderPage(c echo.Context, name string, oc oauth2.Config, data echo.Map) error {
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

	s := echosession.FromContext(c)
	addCurrentUser(s, data)
	data["LoginURL"] = oc.AuthCodeURL(random.String(32))
	return c.Render(http.StatusOK, name, data)
}

func addCurrentUser(s session.Store, params echo.Map) {
	i, iok := s.Get("current_user_id")
	u, uok := s.Get("current_user_name")
	if iok && uok {
		params["CurrentUser"] = auth.User{
			ID:       i.(string),
			Username: u.(string),
		}
	}
}
