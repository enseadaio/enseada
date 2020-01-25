// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"html/template"
	"io"

	rice "github.com/GeertJohan/go.rice"
	"github.com/foolin/goview"
	"github.com/foolin/goview/supports/gorice"
	"github.com/labstack/echo"
)

type templateRenderer struct {
	*goview.ViewEngine
}

func newGoViewRenderer() *templateRenderer {
	box := rice.MustFindBox("../../web/templates")
	gv := gorice.NewWithConfig(box, goview.Config{
		Root:         "views",
		Extension:    ".html",
		Master:       "layouts/master",
		Partials:     []string{"partials/navbar", "partials/errors"},
		Funcs:        make(template.FuncMap),
		DisableCache: false,
		Delims:       goview.Delims{Left: "{{", Right: "}}"},
	})
	return &templateRenderer{ViewEngine: gv}

}

func (t *templateRenderer) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.ViewEngine.RenderWriter(w, name, data)
}
