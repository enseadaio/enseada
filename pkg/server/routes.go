package server

import (
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/labstack/echo"
)

func routes(e *echo.Echo, mvn *maven.Maven) {
	mountMaven(e, mvn)
	mountHealthCheck(e)
	mountUI(e)
}
