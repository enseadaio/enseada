package server

import (
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/enseadaio/enseada/pkg/repo"
	"github.com/labstack/echo"
)

func routes(e *echo.Echo, r *repo.Service, mvn *maven.Maven) {
	mountRepoV1(e, r, mvn)
	mountMaven(e, mvn)
}
