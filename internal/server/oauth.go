package server

import (
	"github.com/enseadaio/enseada/pkg/auth"
	echosession "github.com/go-session/echo-session"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/random"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
	"github.com/ory/fosite/token/jwt"
	"net/http"
	"time"
)

func mountOauth(e *echo.Echo, oauth fosite.OAuth2Provider) {
	g := e.Group("/oauth")
	g.GET("/authorize", authorizationPage())
	g.POST("/authorize", authorize(oauth))
	g.POST("/token", token(oauth))
	g.POST("/token/introspect", introspect(oauth))
}

func authorizationPage() echo.HandlerFunc {
	return func(c echo.Context) error {
		session := echosession.FromContext(c)
		e, ok := session.Get("error")
		params := echo.Map{
			"ClientID":     QueryWithDefault(c, "client_id", ""),
			"RedirectURI":  QueryWithDefault(c, "redirect_uri", ""),
			"State":        QueryWithDefault(c, "state", random.String(32)),
			"Scope":        QueryWithDefault(c, "scope", ""),
			"Audience":     QueryWithDefault(c, "audience", ""),
			"ResponseType": QueryWithDefault(c, "response_type", "code"),
		}
		if ok {
			params["Error"] = e
		}

		return c.Render(http.StatusOK, "login", params)
	}
}

func authorize(oauth fosite.OAuth2Provider) echo.HandlerFunc {
	return func(c echo.Context) error {
		req := c.Request()
		resw := c.Response()
		ctx := req.Context()

		//username := strings.TrimSpace(req.FormValue("username"))
		//password := strings.TrimSpace(req.FormValue("password"))

		//u, err := oauth.Authenticate(ctx, username, password)
		//if err != nil {
		//	if strings.Contains(req.Header.Get("accept"), "html") {
		//		session := echosession.FromContext(c)
		//		session.Set("error", "Invalid username of password")
		//		if err := session.Save(); err != nil {
		//			return err
		//		}
		//		return c.Redirect(http.StatusSeeOther, c.Request().Header.Get("Referer"))
		//	}
		//
		//	return c.JSON(http.StatusUnauthorized, echo.Map{
		//		"error":   "unauthorized",
		//		"message": "invalid username or password",
		//	})
		//}

		session := newSession(&auth.User{})
		ar, err := oauth.NewAuthorizeRequest(ctx, req)
		if err != nil {
			c.Logger().Error(err)
			oauth.WriteAuthorizeError(resw, ar, err)
			return nil
		}

		for _, scope := range ar.GetRequestedScopes() {
			ar.GrantScope(scope)
		}

		res, err := oauth.NewAuthorizeResponse(ctx, ar, session)
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

		session := newSession(nil)
		c.Logger().Info(req)
		ar, err := oauth.NewAccessRequest(ctx, req, session)
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
		session := newSession(nil)

		ir, err := oauth.NewIntrospectionRequest(ctx, req, session)
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
