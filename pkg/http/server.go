// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"net/http"

	rice "github.com/GeertJohan/go.rice"
	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	authv1beta1api "github.com/enseadaio/enseada/internal/auth/v1beta1"
	"github.com/enseadaio/enseada/internal/maven"
	mavenv1beta1api "github.com/enseadaio/enseada/internal/maven/v1beta1"
	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/enseadaio/enseada/pkg/errare"
	"github.com/enseadaio/enseada/pkg/log"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/go-kivik/kivik"
	eware "github.com/labstack/echo/middleware"
	"github.com/ory/fosite"
	"go.opencensus.io/plugin/ochttp"
	goauth "golang.org/x/oauth2"

	"github.com/labstack/echo"
)

type Server struct {
	*echo.Echo
	logger          log.Logger
	errorHandler    errare.Handler
	store           *auth.Store
	oauthProvider   fosite.OAuth2Provider
	data            *kivik.Client
	enforcer        *casbin.Enforcer
	oauthClient     *goauth.Config
	metricsRegistry auth.MetricsRegistry
	mvn             *maven.Maven
	skb             []byte
}

func createServer(deps Deps) *Server {
	logger := deps.Logger
	data := deps.Data
	errh := deps.ErrorHandler
	oc := deps.OAuthClient
	skb := deps.SecretKeyBase
	store := deps.Store
	op := deps.OAuthProvider
	enf := deps.Enforcer
	mr := deps.MetricsRegistry
	mvn := deps.Maven

	e := echo.New()

	e.Logger = &EchoLogger{
		Logger: logger,
		pfx:    "echo",
	}
	e.HideBanner = true
	e.HTTPErrorHandler = newErrorHandler(errh)
	e.Renderer = newGoViewRenderer()

	return &Server{
		Echo:            e,
		logger:          logger,
		data:            data,
		errorHandler:    errh,
		oauthClient:     oc,
		skb:             skb,
		store:           store,
		oauthProvider:   op,
		enforcer:        enf,
		metricsRegistry: mr,
		mvn:             mvn,
	}
}

func middlewares(s *Server) {
	e := s.Echo
	l := s.logger
	st := s.store
	op := s.oauthProvider
	errh := s.errorHandler

	e.Use(eware.Recover())
	e.Use(eware.CORS())
	e.Use(eware.RequestID())
	e.Use(eware.Logger())
	e.Use(eware.SecureWithConfig(eware.SecureConfig{
		XSSProtection:      "1; mode=block",
		ContentTypeNosniff: "nosniff",
		XFrameOptions:      "SAMEORIGIN",
		HSTSMaxAge:         3600,
	}))
	e.Use(eware.GzipWithConfig(eware.GzipConfig{
		Level: 5,
		Skipper: func(c echo.Context) bool {
			return c.Path() == "/metrics"
		},
	}))
	e.Use(echo.WrapMiddleware(middleware.AuthorizationHeader(l, st, op, errh)))

	e.Pre(eware.RemoveTrailingSlashWithConfig(
		eware.TrailingSlashConfig{
			RedirectCode: http.StatusMovedPermanently,
		}))
	paths := fosite.Arguments{"/health", "/metrics"}
	e.Pre(echo.WrapMiddleware(func(base http.Handler) http.Handler {
		return &ochttp.Handler{
			Handler: base,
			IsHealthEndpoint: func(r *http.Request) bool {
				return paths.Has(r.URL.Path)
			},
		}
	}))
}

func routes(s *Server) {
	e := s.Echo
	l := s.logger
	oc := s.oauthClient
	op := s.oauthProvider
	st := s.store
	enf := s.enforcer
	mr := s.metricsRegistry
	data := s.data
	mvn := s.mvn
	skb := s.skb

	sm := middleware.Session(skb)

	// UI
	ui := &UIHandler{oc: oc}
	e.GET("/", ui.root)

	staticHandler := http.FileServer(rice.MustFindBox("../../web/static").HTTPBox())
	e.GET("/static/*", echo.WrapHandler(http.StripPrefix("/static/", staticHandler)))

	assetHandler := http.FileServer(rice.MustFindBox("../../web/assets").HTTPBox())
	e.GET("/assets/*", echo.WrapHandler(http.StripPrefix("/assets/", assetHandler)))

	u := e.Group("/ui")
	u.Use(sm)
	u.Use(eware.CSRF())
	u.GET("", ui.home)
	u.GET("/callback", ui.callback)
	u.GET("/error", ui.errorPage)

	// OAuth
	oauth := &OAuthHandler{oauthProvider: op, store: st}
	og := e.Group("/oauth")
	og.Use(sm)
	og.GET("/authorize", oauth.authorizationPage)
	og.GET("/consent", oauth.consentPage)
	og.POST("/authorize", oauth.authorize)
	og.POST("/token", oauth.token)
	og.POST("/revoke", oauth.revoke)
	og.POST("/token/introspect", oauth.introspect)

	// Maven
	m := &MavenHandler{mvn: mvn, enforcer: enf}
	mg := e.Group("/maven2")
	mg.GET("/*", m.getFile)
	mg.PUT("/*", m.putFile)

	// Twirp
	aclApi := authv1beta1api.NewAclAPI(l, enf)
	aclhandler := authv1beta1.NewAclAPIServer(aclApi, nil)
	e.Any(aclhandler.PathPrefix()+"*", echo.WrapHandler(aclhandler))

	oclientsApi := authv1beta1api.NewOAuthClientsAPI(l, enf, st)
	oclientshandler := authv1beta1.NewOAuthClientsAPIServer(oclientsApi, nil)
	e.Any(oclientshandler.PathPrefix()+"*", echo.WrapHandler(oclientshandler))

	usersApi := authv1beta1api.NewUsersAPI(l, enf, st, mr)
	usershandler := authv1beta1.NewUsersAPIServer(usersApi, nil)
	e.Any(usershandler.PathPrefix()+"*", echo.WrapHandler(usershandler))

	mvnApi := mavenv1beta1api.NewMavenAPI(mvn, enf)
	mvnhandler := mavenv1beta1.NewMavenAPIServer(mvnApi, nil)
	e.Any(mvnhandler.PathPrefix()+"*", echo.WrapHandler(mvnhandler))

	// Olly
	olly := &ObservabilityHandler{data: data}
	e.GET("/health", olly.health)
	e.GET("/metrics", olly.metrics)
}
