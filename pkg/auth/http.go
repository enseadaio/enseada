// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"errors"
	"net/http"
	"strings"

	"github.com/labstack/gommon/random"
	"go.uber.org/multierr"

	"github.com/enseadaio/enseada/internal/cachecontrol"

	"github.com/enseadaio/enseada/pkg/errare"

	"github.com/enseadaio/enseada/internal/middleware"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	authv1beta1api "github.com/enseadaio/enseada/internal/auth/v1beta1"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
)

func mountRoutes(e *echo.Echo, s *auth.Store, op fosite.OAuth2Provider, enf *casbin.Enforcer, sm echo.MiddlewareFunc, errh errare.Handler, m *auth.Metrics) error {
	e.Use(echo.WrapMiddleware(middleware.AuthorizationHeader(s.Logger, s, op, errh)))

	g := e.Group("/oauth")
	g.Use(sm)
	g.GET("/authorize", authorizationPage())
	g.POST("/authorize", authorize(op, s))
	g.POST("/token", token(op, s))
	g.POST("/revoke", revoke(op))
	g.POST("/token/introspect", introspect(op))

	acl := authv1beta1api.NewAclAPI(s.Logger, enf)
	aclhandler := authv1beta1.NewAclAPIServer(acl, nil)
	e.Any(aclhandler.PathPrefix()+"*", echo.WrapHandler(aclhandler))

	oclients := authv1beta1api.NewOAuthClientsAPI(s.Logger, enf, s)
	oclientshandler := authv1beta1.NewOAuthClientsAPIServer(oclients, nil)
	e.Any(oclientshandler.PathPrefix()+"*", echo.WrapHandler(oclientshandler))

	users := authv1beta1api.NewUsersAPI(s.Logger, enf, s, m)
	usershandler := authv1beta1.NewUsersAPIServer(users, nil)
	e.Any(usershandler.PathPrefix()+"*", echo.WrapHandler(usershandler))
	return nil
}

type LoginQueryParams struct {
	ClientID     string `query:"client_id"`
	RedirectURI  string `query:"redirect_uri"`
	State        string `query:"state"`
	Scope        string `query:"scope"`
	Audience     string `query:"audience"`
	ResponseType string `query:"response_type"`
}

func authorizationPage() echo.HandlerFunc {
	return func(c echo.Context) error {
		s := session.Default(c)

		p := new(LoginQueryParams)
		if err := c.Bind(p); err != nil {
			return err
		}

		var err error
		if p.ClientID == "" {
			err = multierr.Append(err, errors.New("client_id is required"))
		}
		if p.RedirectURI == "" {
			err = multierr.Append(err, errors.New("redirect_uri is required"))
		}
		if p.State == "" {
			p.State = random.String(64)
		}
		if p.Scope == "" {
			err = multierr.Append(err, errors.New("scope is required"))
		}
		if p.Audience == "" {
			p.Audience = "enseada"
		}
		if p.ResponseType == "" {
			err = multierr.Append(err, errors.New("response_type is required"))
		}
		errs := multierr.Errors(err)
		if len(errs) > 0 {
			for _, e := range errs {
				s.AddFlash(e.Error(), "errors")
			}
			if err := s.Save(); err != nil {
				return err
			}
			return c.Redirect(http.StatusTemporaryRedirect, "/ui/error")
		}

		params := echo.Map{
			"ClientID":     p.ClientID,
			"RedirectURI":  p.RedirectURI,
			"State":        p.State,
			"Scope":        p.Scope,
			"Audience":     p.Audience,
			"ResponseType": p.ResponseType,
		}

		sc := http.StatusOK
		e := s.Flashes("errors")
		if len(e) > 0 {
			params["Errors"] = e
			sc = http.StatusBadRequest
		}
		ue := s.Flashes("UsernameError")
		if len(ue) > 0 {
			params["UsernameError"] = ue[0]
			sc = http.StatusUnprocessableEntity
		}

		pe := s.Flashes("PasswordError")
		if len(pe) > 0 {
			params["PasswordError"] = pe[0]
			sc = http.StatusUnprocessableEntity
		}

		return c.Render(sc, "login", params)
	}
}

func authorize(oauth fosite.OAuth2Provider, store *auth.Store) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response()
		ctx := req.Context()
		s := session.Default(c)

		ar, err := oauth.NewAuthorizeRequest(ctx, req)
		if err != nil {
			rfce := fosite.ErrorToRFC6749Error(err)
			rfce = rfce.WithDescription(rfce.Hint)
			c.Logger().Error(rfce)
			oauth.WriteAuthorizeError(resw, ar, rfce)
			return nil
		}

		for _, scope := range ar.GetRequestedScopes() {
			ar.GrantScope(scope)
		}

		username := strings.TrimSpace(req.FormValue("username"))
		password := strings.TrimSpace(req.FormValue("password"))
		accepsHTML := strings.Contains(req.Header.Get("accept"), "html")

		formErrs := echo.Map{}
		if username == "" {
			formErrs["UsernameError"] = "username cannot be blank"
		}

		if password == "" {
			formErrs["PasswordError"] = "password cannot be blank"
		}

		if len(formErrs) > 0 {
			s.Clear()
			if accepsHTML {
				s.AddFlash(formErrs["UsernameError"], "UsernameError")
				s.AddFlash(formErrs["PasswordError"], "PasswordError")
				if err := s.Save(); err != nil {
					return err
				}
				return c.Redirect(http.StatusSeeOther, req.Header.Get("Referer"))
			} else {
				return echo.NewHTTPError(http.StatusBadRequest, formErrs["UsernameError"], formErrs["PasswordError"])
			}
		}

		err = store.Authenticate(ctx, username, password)
		if err != nil {
			s.Clear()
			if accepsHTML {
				s.AddFlash("Invalid username of password", "errors")
				if err := s.Save(); err != nil {
					return err
				}
				return c.Redirect(http.StatusSeeOther, req.Header.Get("Referer"))
			}
			oauth.WriteAuthorizeError(resw, ar, fosite.ErrAccessDenied)
			return nil
		}

		u, err := store.GetUser(ctx, username)
		if err != nil {
			return err
		}

		os := auth.NewSession(u)
		res, err := oauth.NewAuthorizeResponse(ctx, ar, os)
		if err != nil {
			rfce := fosite.ErrorToRFC6749Error(err)
			rfce = rfce.WithDescription(rfce.Hint)
			c.Logger().Error(rfce)
			oauth.WriteAuthorizeError(resw, ar, rfce)
			return nil
		}

		oauth.WriteAuthorizeResponse(resw, ar, res)
		return nil
	}
}

func token(oauth fosite.OAuth2Provider, store *auth.Store) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response()
		ctx := req.Context()

		cc := cachecontrol.NoStore(true)
		cc.Write(resw.Writer)

		os := auth.NewSession(nil)
		c.Logger().Debug(req)
		ar, err := oauth.NewAccessRequest(ctx, req, os)
		if err != nil {
			rfce := fosite.ErrorToRFC6749Error(err)
			if strings.Contains(rfce.Debug, "password") {
				c.Logger().Error("authentication failed")
				oauth.WriteAccessError(resw, ar, fosite.ErrAccessDenied)
				return nil

			}
			rfce = rfce.WithDescription(rfce.Hint)
			c.Logger().Error(rfce)
			oauth.WriteAccessError(resw, ar, rfce)
			return nil
		}

		for _, scope := range ar.GetRequestedScopes() {
			if fosite.WildcardScopeStrategy(ar.GetClient().GetScopes(), scope) {
				ar.GrantScope(scope)
			}
		}

		// If this is a password grant, populate the session.
		if ar.GetGrantTypes().Exact("password") {
			username := strings.TrimSpace(req.FormValue("username"))
			u, err := store.GetUser(ctx, username)
			if err != nil {
				return err
			}

			ar.SetSession(auth.NewSession(u))
		}

		res, err := oauth.NewAccessResponse(ctx, ar)
		if err != nil {
			rfce := fosite.ErrorToRFC6749Error(err)
			rfce = rfce.WithDescription(rfce.Hint)
			c.Logger().Error(rfce)
			oauth.WriteAccessError(resw, ar, rfce)
			return nil
		}

		oauth.WriteAccessResponse(resw, ar, res)
		return nil
	}
}

func revoke(oauth fosite.OAuth2Provider) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response().Writer
		ctx := req.Context()

		err := oauth.NewRevocationRequest(ctx, req)
		oauth.WriteRevocationResponse(resw, err)
		return nil
	}
}

func introspect(oauth fosite.OAuth2Provider) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		req := c.Request()
		resw := c.Response().Writer
		os := auth.NewSession(nil)

		ir, err := oauth.NewIntrospectionRequest(ctx, req, os)
		if err != nil {
			rfce := fosite.ErrorToRFC6749Error(err)
			rfce = rfce.WithDescription(rfce.Hint)
			c.Logger().Error(rfce)
			oauth.WriteIntrospectionError(resw, rfce)
			return nil
		}

		oauth.WriteIntrospectionResponse(resw, ir)
		return nil
	}
}
