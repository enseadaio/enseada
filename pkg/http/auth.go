package http

import (
	"github.com/enseadaio/enseada/internal/utils"
	"github.com/enseadaio/enseada/pkg/auth"
	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/random"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
	"github.com/ory/fosite/token/jwt"
	"net/http"
	"strings"
	"time"
)

func mountAuth(e *echo.Echo, oauth fosite.OAuth2Provider, store *auth.Store, sm echo.MiddlewareFunc) {
	g := e.Group("/oauth")
	g.Use(sm)
	g.GET("/authorize", authorizationPage())
	g.POST("/authorize", authorize(oauth, store))
	g.POST("/token", token(oauth))
	g.POST("/token/introspect", introspect(oauth))
}

func authorizationPage() echo.HandlerFunc {
	return func(c echo.Context) error {
		s := session.Default(c)
		e := s.Flashes("errors")
		params := echo.Map{
			"ClientID":     utils.QueryWithDefault(c, "client_id", ""),
			"RedirectURI":  utils.QueryWithDefault(c, "redirect_uri", ""),
			"State":        utils.QueryWithDefault(c, "state", random.String(32)),
			"Scope":        utils.QueryWithDefault(c, "scope", ""),
			"Audience":     utils.QueryWithDefault(c, "audience", ""),
			"ResponseType": utils.QueryWithDefault(c, "response_type", "code"),
		}
		if len(e) > 0 {
			params["Errors"] = e
		}

		return c.Render(http.StatusOK, "login", params)
	}
}

func authorize(oauth fosite.OAuth2Provider, store *auth.Store) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response()
		ctx := req.Context()

		username := strings.TrimSpace(req.FormValue("username"))
		password := strings.TrimSpace(req.FormValue("password"))

		err := store.Authenticate(ctx, username, password)
		if err != nil {
			if strings.Contains(req.Header.Get("accept"), "html") {
				s := session.Default(c)
				s.AddFlash("Invalid username of password", "errors")
				if err := s.Save(); err != nil {
					return err
				}
				return c.Redirect(http.StatusSeeOther, c.Request().Header.Get("Referer"))
			}

			return c.JSON(http.StatusUnauthorized, echo.Map{
				"error":   "unauthorized",
				"message": "invalid username or password",
			})
		}

		u, err := store.FindByUsername(ctx, username)
		if err != nil {
			return err
		}

		os := newSession(u)
		ar, err := oauth.NewAuthorizeRequest(ctx, req)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteAuthorizeError(resw, ar, err)
			return nil
		}

		for _, scope := range ar.GetRequestedScopes() {
			ar.GrantScope(scope)
		}

		res, err := oauth.NewAuthorizeResponse(ctx, ar, os)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteAuthorizeError(resw, ar, err)
			return nil
		}

		oauth.WriteAuthorizeResponse(resw, ar, res)
		return nil
	}
}

func token(oauth fosite.OAuth2Provider) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response()
		ctx := req.Context()

		os := newSession(nil)
		c.Logger().Info(req)
		ar, err := oauth.NewAccessRequest(ctx, req, os)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteAccessError(resw, ar, err)
			return nil
		}

		for _, scope := range ar.GetRequestedScopes() {
			ar.GrantScope(scope)
		}

		res, err := oauth.NewAccessResponse(ctx, ar)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteAccessError(resw, ar, err)
			return nil
		}

		oauth.WriteAccessResponse(resw, ar, res)
		return nil
	}
}

func introspect(oauth fosite.OAuth2Provider) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		req := c.Request()
		resw := c.Response().Writer
		os := newSession(nil)

		ir, err := oauth.NewIntrospectionRequest(ctx, req, os)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteIntrospectionError(resw, err)
			return nil
		}

		oauth.WriteIntrospectionResponse(resw, ir)
		return nil
	}
}
func newSession(u *auth.User) fosite.Session {
	if u == nil {
		return &openid.DefaultSession{
			Claims: &jwt.IDTokenClaims{
				Issuer:      "enseada",
				Subject:     "",
				Audience:    []string{"enseada"},
				Nonce:       "",
				ExpiresAt:   time.Now().Add(time.Hour * 6),
				IssuedAt:    time.Now(),
				RequestedAt: time.Now(),
				AuthTime:    time.Now(),
			},
			Username: "",
			Subject:  "",
		}
	}

	return &openid.DefaultSession{
		Claims: &jwt.IDTokenClaims{
			Issuer:      "enseada",
			Subject:     u.ID,
			Audience:    []string{"enseada"},
			Nonce:       "",
			ExpiresAt:   time.Now().Add(time.Hour * 6),
			IssuedAt:    time.Now(),
			RequestedAt: time.Now(),
			AuthTime:    time.Now(),
			Extra: echo.Map{
				"username": u.Username,
			},
		},
		Username: u.Username,
		Subject:  u.ID,
	}
}
