// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"errors"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/labstack/gommon/random"
	"go.uber.org/multierr"

	"github.com/enseadaio/enseada/internal/cachecontrol"

	"github.com/enseadaio/enseada/internal/auth"
	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
)

type OAuthHandler struct {
	oauthProvider fosite.OAuth2Provider
	store         *auth.Store
}

type LoginQueryParams struct {
	ClientID     string `form:"client_id" query:"client_id"`
	RedirectURI  string `form:"redirect_uri" query:"redirect_uri"`
	State        string `form:"state" query:"state"`
	Scope        string `form:"scope" query:"scope"`
	Audience     string `form:"audience" query:"audience"`
	ResponseType string `form:"response_type" query:"response_type"`
}

func (p *LoginQueryParams) QueryString() string {
	v := make(url.Values)

	v.Add("client_id", p.ClientID)
	v.Add("redirect_uri", p.RedirectURI)
	v.Add("state", p.State)
	v.Add("scope", p.Scope)
	v.Add("audience", p.Audience)
	v.Add("response_type", p.ResponseType)

	return v.Encode()
}

func (oh *OAuthHandler) authorizationPage(c echo.Context) error {
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
		"Title":        "Login",
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

	if err := s.Save(); err != nil {
		return err
	}
	return c.Render(sc, "login", params)
}

func (oh *OAuthHandler) consentPage(c echo.Context) error {
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
		return c.Redirect(http.StatusTemporaryRedirect, "/oauth/authorize")
	}

	scopes := strings.Split(p.Scope, " ")
	params := echo.Map{
		"Title":        "Login",
		"ClientID":     p.ClientID,
		"RedirectURI":  p.RedirectURI,
		"State":        p.State,
		"Scope":        p.Scope,
		"Audience":     p.Audience,
		"ResponseType": p.ResponseType,
		"Scopes":       scopes,
	}

	sc := http.StatusOK
	e := s.Flashes("errors")
	if len(e) > 0 {
		params["Errors"] = e
		sc = http.StatusBadRequest
	}
	if err := s.Save(); err != nil {
		return err
	}
	return c.Render(sc, "consent", params)
}

func (oh *OAuthHandler) authorize(c echo.Context) error {
	op := oh.oauthProvider
	st := oh.store

	req := c.Request()
	resw := c.Response()
	ctx := req.Context()
	s := session.Default(c)

	ar, err := op.NewAuthorizeRequest(ctx, req)
	if err != nil {
		rfce := fosite.ErrorToRFC6749Error(err)
		rfce = rfce.WithDescription(rfce.Hint)
		c.Logger().Error(rfce)
		s.Clear()
		if err := s.Save(); err != nil {
			return err
		}
		op.WriteAuthorizeError(resw, ar, rfce)
		return nil
	}

	uid := s.Get("current-user-id")
	if uid == nil {
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
				if err := s.Save(); err != nil {
					return err
				}
				return echo.NewHTTPError(http.StatusBadRequest, formErrs["UsernameError"], formErrs["PasswordError"])
			}
		}

		err = st.Authenticate(ctx, username, password)
		if err != nil {
			s.Clear()
			if accepsHTML {
				s.AddFlash("Invalid username of password", "errors")
				if err := s.Save(); err != nil {
					return err
				}
				return c.Redirect(http.StatusSeeOther, req.Header.Get("Referer"))
			}
			if err := s.Save(); err != nil {
				return err
			}
			op.WriteAuthorizeError(resw, ar, fosite.ErrAccessDenied)
			return nil
		}
		s.Set("current-user-id", username)
		if err := s.Save(); err != nil {
			return err
		}
		uid = username
	}

	u, err := st.GetUser(ctx, uid.(string))
	if err != nil {
		return err
	}

	if len(u.Consent) == 0 {
		u.Consent = make(map[string]auth.UserConsent)
	}

	cons := u.Consent[ar.GetClient().GetID()]
	if req.FormValue("consent") == "" && (c.QueryParam("prompt") == "consent" || cons.ConsentGivenAt.IsZero() || !fosite.Arguments(cons.Scopes).Has(ar.GetRequestedScopes()...)) {
		p := new(LoginQueryParams)
		if err := c.Bind(p); err != nil {
			return err
		}
		u.Consent[ar.GetClient().GetID()] = auth.UserConsent{
			ConsentGivenAt: time.Time{},
		}
		if err := st.CreateUser(ctx, u); err != nil {
			return err
		}
		if err := s.Save(); err != nil {
			return err
		}
		return c.Redirect(http.StatusSeeOther, "/oauth/consent?"+p.QueryString())
	}

	for _, scope := range ar.GetRequestedScopes() {
		if fosite.WildcardScopeStrategy(ar.GetClient().GetScopes(), scope) {
			ar.GrantScope(scope)
		}
	}

	if cons.ConsentGivenAt.IsZero() {
		u.Consent[ar.GetClient().GetID()] = auth.UserConsent{
			Scopes:         ar.GetGrantedScopes(),
			ConsentGivenAt: time.Now(),
		}
		if err := st.UpdateUser(ctx, u); err != nil {
			return err
		}
	}

	os := auth.NewSession(u)
	res, err := op.NewAuthorizeResponse(ctx, ar, os)
	if err != nil {
		rfce := fosite.ErrorToRFC6749Error(err)
		rfce = rfce.WithDescription(rfce.Hint)
		c.Logger().Error(rfce)
		s.Clear()
		if err := s.Save(); err != nil {
			return err
		}
		op.WriteAuthorizeError(resw, ar, rfce)
		return nil
	}

	s.Clear()
	if err := s.Save(); err != nil {
		return err
	}

	op.WriteAuthorizeResponse(resw, ar, res)
	return nil
}

func (oh *OAuthHandler) token(c echo.Context) error {
	op := oh.oauthProvider
	st := oh.store

	req := c.Request()
	resw := c.Response()
	ctx := req.Context()

	cc := cachecontrol.NoStore(true)
	cc.Write(resw.Writer)

	os := auth.NewSession(nil)
	c.Logger().Debug(req)
	ar, err := op.NewAccessRequest(ctx, req, os)
	if err != nil {
		rfce := fosite.ErrorToRFC6749Error(err)
		if strings.Contains(rfce.Debug, "password") {
			c.Logger().Error("authentication failed")
			op.WriteAccessError(resw, ar, fosite.ErrAccessDenied)
			return nil

		}
		rfce = rfce.WithDescription(rfce.Hint)
		c.Logger().Error(rfce)
		op.WriteAccessError(resw, ar, rfce)
		return nil
	}

	for _, scope := range ar.GetRequestedScopes() {
		if fosite.WildcardScopeStrategy(ar.GetClient().GetScopes(), scope) {
			ar.GrantScope(scope)
		} else {
			op.WriteAccessError(resw, ar, fosite.ErrInvalidScope.WithHintf(`The OAuth 2.0 Client is not allowed to request scope "%s".`, scope))
			return nil
		}
	}

	// If this is a password grant, populate the session.
	if ar.GetGrantTypes().Exact("password") {
		username := strings.TrimSpace(req.FormValue("username"))
		u, err := st.GetUser(ctx, username)
		if err != nil {
			return err
		}

		ar.SetSession(auth.NewSession(u))
	}

	res, err := op.NewAccessResponse(ctx, ar)
	if err != nil {
		rfce := fosite.ErrorToRFC6749Error(err)
		rfce = rfce.WithDescription(rfce.Hint)
		c.Logger().Error(rfce)
		op.WriteAccessError(resw, ar, rfce)
		return nil
	}

	op.WriteAccessResponse(resw, ar, res)
	return nil
}

func (oh *OAuthHandler) revoke(c echo.Context) error {
	op := oh.oauthProvider

	req := c.Request()
	resw := c.Response().Writer
	ctx := req.Context()

	err := op.NewRevocationRequest(ctx, req)
	op.WriteRevocationResponse(resw, err)
	return nil
}

func (oh *OAuthHandler) introspect(c echo.Context) error {
	op := oh.oauthProvider

	ctx := c.Request().Context()
	req := c.Request()
	resw := c.Response().Writer
	os := auth.NewSession(nil)

	ir, err := op.NewIntrospectionRequest(ctx, req, os)
	if err != nil {
		rfce := fosite.ErrorToRFC6749Error(err)
		rfce = rfce.WithDescription(rfce.Hint)
		c.Logger().Error(rfce)
		op.WriteIntrospectionError(resw, rfce)
		return nil
	}

	op.WriteIntrospectionResponse(resw, ir)
	return nil
}
