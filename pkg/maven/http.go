// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"strings"
	"time"

	"github.com/enseadaio/enseada/internal/cachecontrol"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/ctxutils"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/internal/scope"
	"github.com/enseadaio/enseada/internal/utils"

	"github.com/casbin/casbin/v2"

	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/maven"
	mavenv1beta1api "github.com/enseadaio/enseada/internal/maven/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/ory/fosite"

	"github.com/labstack/echo"
)

func mountRoutes(e *echo.Echo, m *maven.Maven, s *auth.Store, op fosite.OAuth2Provider, enf *casbin.Enforcer) {
	g := e.Group("/maven2")

	g.GET("/*", getMaven(m, enf))
	g.PUT("/*", storeMaven(m, enf))

	mvnsvc := mavenv1beta1api.Service{
		Maven:    m,
		Enforcer: enf,
	}
	mvnhandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	e.Any(mvnhandler.PathPrefix()+"*", echo.WrapHandler(mvnhandler))
}

func getMaven(mvn *maven.Maven, enf *casbin.Enforcer) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()

		uid, ok := ctxutils.CurrentUserID(ctx)
		if !ok {
			return c.NoContent(http.StatusUnauthorized)
		}

		scopes, _ := ctxutils.Scopes(ctx)
		if !fosite.WildcardScopeStrategy(scopes, scope.MavenFileRead) {
			return c.JSON(http.StatusForbidden, utils.HTTPError(http.StatusForbidden, "insufficient scopes"))
		}

		path := strings.TrimPrefix(c.Request().RequestURI, "/")
		c.Logger().Infof("Loading file from %s", path)
		file, err := mvn.GetFile(ctx, path)
		if err != nil {
			return err
		}

		if file == nil {
			c.Logger().Warnf("No file found at %s", path)
			return c.NoContent(http.StatusNotFound)

		}

		gr := guid.New(couch.MavenDB, file.Repo.ID, couch.KindRepository)
		can, err := enf.Enforce(uid, gr.String(), "read")
		if err != nil {
			return err
		}

		if !can {
			return c.NoContent(http.StatusNotFound)
		}

		c.Logger().Info("File found")
		body, err := file.Content()
		if err != nil {
			return err
		}

		lm, err := file.LastModified()
		if err != nil {
			return err
		}

		sha, err := file.Checksum()
		if err != nil {
			return err
		}
		cs := fmt.Sprintf("%x", sha)

		ret := c.Request().Header.Get("etag")
		if ret != "" && ret == cs {
			return c.NoContent(http.StatusNotModified)
		}

		cc := &cachecontrol.Config{
			NoTransform:  true,
			LastModified: lm,
		}

		if file.Version != nil && !file.Version.IsSnapshot() {
			cc.Immutable = true
		} else {
			cc.MaxAge = 30 * time.Second
		}

		if cc.Immutable {
			cc.ETag = cs
		} else {
			cc.ETag = fmt.Sprintf("W/%s", cs)
		}
		cc.Write(c.Response().Writer)
		return c.Blob(http.StatusOK, "application/octet-stream", body)
	}
}

func storeMaven(mvn *maven.Maven, enf *casbin.Enforcer) echo.HandlerFunc {
	return func(c echo.Context) error {
		ctx := c.Request().Context()

		uid, ok := ctxutils.CurrentUserID(ctx)
		if !ok {
			return c.NoContent(http.StatusUnauthorized)
		}

		scopes, _ := ctxutils.Scopes(ctx)
		if !fosite.WildcardScopeStrategy(scopes, scope.MavenFileWrite) {
			return c.JSON(http.StatusForbidden, utils.HTTPError(http.StatusForbidden, "insufficient scopes"))
		}

		path := strings.TrimPrefix(c.Request().RequestURI, "/maven2/")
		c.Logger().Infof("attempting storing Maven artifact at %s", path)
		body, err := ioutil.ReadAll(c.Request().Body)
		if err != nil {
			return err
		}

		repos, err := mvn.ListRepos(ctx, couch.Query{})
		if err != nil {
			return err
		}

		var repo *maven.Repo
		for _, r := range repos {
			if strings.HasPrefix(path, r.StoragePath) {
				repo = r
				break
			}
		}

		if repo == nil {
			return c.NoContent(http.StatusNotFound)
		}

		gr := guid.New(couch.MavenDB, repo.ID, couch.KindRepository)
		can, err := enf.Enforce(uid, gr.String(), "write")
		if err != nil {
			return err
		}

		if !can {
			return c.NoContent(http.StatusNotFound)
		}

		file, err := mvn.PutFileInRepo(ctx, repo, path, body)
		if err != nil {
			c.Logger().Error(err)
			if err == maven.ErrRepoNotFound {
				return c.String(http.StatusNotFound, err.Error())
			}
			if strings.Contains(err.Error(), "is immutable") {
				return c.String(http.StatusConflict, err.Error())
			}
			return err
		}

		cc := &cachecontrol.Config{
			NoCache: true,
			MaxAge:  0,
		}
		if file.Version != nil {
			cc.Immutable = !file.Version.IsSnapshot()
		}
		cc.Write(c.Response().Writer)
		c.Logger().Info("stored Maven artifact %s at %s", file.Filename, path)
		return c.NoContent(http.StatusCreated)
	}
}
