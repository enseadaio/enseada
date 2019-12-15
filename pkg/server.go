package enseada

import (
	"github.com/casbin/casbin/v2"
	"github.com/chartmuseum/storage"
	authsvcv1beta1 "github.com/enseadaio/enseada/internal/authsvc/v1beta1"
	"github.com/enseadaio/enseada/internal/maven"
	mavensvcv1beta1 "github.com/enseadaio/enseada/internal/mavensvc/v1beta1"
	"github.com/enseadaio/enseada/internal/server"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/echo/middleware"
	"github.com/labstack/gommon/log"
	"net/http"
)

type Server struct {
	*echo.Echo
	Maven    *maven.Maven
	Enforcer *casbin.Enforcer
}

func handleErrors(err error, c echo.Context) {
	e := c.JSON(http.StatusInternalServerError, server.HTTPError(http.StatusInternalServerError, err.Error()))
	if e != nil {
		c.Logger().Error(e)
	}
}

func NewServer(client *kivik.Client, storage storage.Backend, en *casbin.Enforcer, opts ...ServerOption) (*Server, error) {
	options := &ServerOptions{
		level: log.INFO,
	}

	for _, opt := range opts {
		err := opt(options)
		if err != nil {
			return nil, err
		}
	}

	e := echo.New()

	e.Logger.SetLevel(options.level)
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

	mvnLog := log.New("maven2")
	mvnLog.SetLevel(options.level)
	mvn, err := maven.New(client, storage, mvnLog)
	if err != nil {
		return nil, err
	}
	return &Server{
		Echo:     e,
		Maven:    mvn,
		Enforcer: en,
	}, nil
}

func (s *Server) Init() {
	mvnsvc := mavensvcv1beta1.Service{Maven: s.Maven}
	mvnHandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	s.Echo.Any(mvnHandler.PathPrefix()+"*", echo.WrapHandler(mvnHandler))

	authLog := log.New("casbin")
	authLog.SetLevel(s.Logger.Level())
	authsvc := authsvcv1beta1.Service{
		Logger:   authLog,
		Enforcer: s.Enforcer,
	}
	authHandler := authv1beta1.NewAclAPIServer(authsvc, nil)
	s.Echo.Any(authHandler.PathPrefix()+"*", echo.WrapHandler(authHandler))

	server.MountRoutes(s.Echo, s.Maven)
}
