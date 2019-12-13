package server

import (
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/enseadaio/enseada/pkg/repo"
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

	e.Use(middleware.Recover())
	e.Use(middleware.CORS())
	e.Use(middleware.RequestID())
	e.Use(middleware.Logger())

	return e
}

func Init(e *echo.Echo, r *repo.Service, mvn *maven.Maven) {
	routes(e, r, mvn)
}
