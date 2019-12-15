package server

import (
	"github.com/enseadaio/enseada/internal/maven"
	mavensvcv1beta1 "github.com/enseadaio/enseada/internal/mavensvc/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/labstack/echo"
	"github.com/labstack/echo/middleware"
	"github.com/labstack/gommon/log"
	"net/http"
)

func handleErrors(err error, c echo.Context) {
	e := c.JSON(http.StatusInternalServerError, HTTPError(http.StatusInternalServerError, err.Error()))
	if e != nil {
		c.Logger().Error(e)
	}
}

func Create(level log.Lvl) *echo.Echo {
	e := echo.New()

	e.Logger.SetLevel(level)
	e.HideBanner = true
	e.HTTPErrorHandler = handleErrors
	e.Renderer = NewGoViewRenderer()

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

	return e
}

func Init(e *echo.Echo, mvn *maven.Maven) {
	mvnsvc := mavensvcv1beta1.Service{Maven: mvn}
	mvnHandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	e.Any(mvnHandler.PathPrefix()+"*", echo.WrapHandler(mvnHandler))

	routes(e, mvn)
}
