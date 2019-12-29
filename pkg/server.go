package enseada

import (
	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/enseadaio/enseada/internal/server"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/labstack/echo"
	"github.com/labstack/echo/middleware"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
	goauth "golang.org/x/oauth2"
	"net/http"
)

type SecretKeyBase []byte
type PublicHost string

type Server struct {
	*echo.Echo
	Maven         *maven.Maven
	Enforcer      *casbin.Enforcer
	OAuthProvider fosite.OAuth2Provider
	OAuthClient   *goauth.Config
	AuthStore     *auth.Store
	SecretKeyBase SecretKeyBase
	PublicHost    PublicHost
}

func handleErrors(err error, c echo.Context) {
	e := c.JSON(http.StatusInternalServerError, server.HTTPError(http.StatusInternalServerError, err.Error()))
	if e != nil {
		c.Logger().Error(e)
	}
}

func NewServer(oauthProvider fosite.OAuth2Provider, oauthClient *goauth.Config, authStore *auth.Store, enforcer *casbin.Enforcer, skb SecretKeyBase, ph PublicHost, lvl log.Lvl) (*Server, error) {
	e := echo.New()

	e.Logger.SetLevel(lvl)
	e.HideBanner = true
	e.HTTPErrorHandler = handleErrors
	e.Renderer = server.NewGoViewRenderer()

	e.Use(middleware.Recover())
	e.Use(middleware.CORS())
	e.Use(middleware.RequestID())
	e.Use(middleware.Logger())
	e.Use(middleware.GzipWithConfig(middleware.GzipConfig{
		Level: 5,
	}))
	e.Pre(middleware.RemoveTrailingSlashWithConfig(
		middleware.TrailingSlashConfig{
			RedirectCode: http.StatusMovedPermanently,
		}))

	return &Server{
		Echo:          e,
		Maven:         nil,
		Enforcer:      enforcer,
		OAuthProvider: oauthProvider,
		OAuthClient:   oauthClient,
		AuthStore:     authStore,
		SecretKeyBase: skb,
		PublicHost:    ph,
	}, nil
}

func (s *Server) Init() {
	//mvnsvc := mavensvcv1beta1.Service{Maven: s.Maven}
	//mvnHandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	//s.Echo.Any(mvnHandler.PathPrefix()+"*", echo.WrapHandler(mvnHandler))
	//
	//authLog := log.New("casbin")
	//authLog.SetLevel(s.Logger.Level())
	//authsvc := authsvcv1beta1.Service{
	//	Logger:   authLog,
	//	Enforcer: s.Enforcer,
	//}
	//authHandler := authv1beta1.NewAclAPIServer(authsvc, nil)
	//s.Echo.Any(authHandler.PathPrefix()+"*", echo.WrapHandler(authHandler))
	//
	//server.MountRoutes(server.RouteParams{
	//	Echo:          s.Echo,
	//	Mvn:           s.Maven,
	//	OAuthProvider: s.OAuthProvider,
	//	OAuthClient:   s.OAuthClient,
	//	PublicHost:    s.PublicHost,
	//	SecretKeyBase: s.SecretKeyBase,
	//})
}
