package server

import (
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/enseadaio/enseada/pkg/repo"
	"github.com/labstack/echo"
	"net/http"
	"strings"
)

func mountRepoV1(e *echo.Echo, r *repo.Service, mvn *maven.Maven) {
	v1 := e.Group("/api/v1")

	v1.GET("/repositories", getReposV1(r))
	v1.POST("/repositories", createRepoV1(mvn))
}

func getReposV1(r *repo.Service) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		repos, err := r.ListRepos(ctx)
		if err != nil {
			return err
		}

		return c.JSON(http.StatusOK, repos)
	}
}

func createRepoV1(mvn *maven.Maven) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()
		body := make(map[string]string)
		if err := c.Bind(&body); err != nil {
			return err
		}

		switch strings.ToLower(body["type"]) {
		case "maven":
			repo := maven.NewRepo(body["group_id"], body["artifact_id"])
			err := mvn.InitRepo(ctx, &repo)
			if err != nil {
				switch err {
				case maven.ErrorRepoAlreadyPresent:
					return c.JSON(http.StatusConflict, HTTPError(http.StatusConflict, err.Error()))
				default:
					return err
				}
			}
			return c.JSON(http.StatusCreated, maven.RepoToHTTPJson(&repo))
		default:
			return c.JSON(http.StatusBadRequest, HTTPError(http.StatusBadRequest, "Unsupported repository type: %s", body["type"]))
		}

	}
}
