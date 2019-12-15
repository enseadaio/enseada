package server

import (
	rice "github.com/GeertJohan/go.rice"
	"github.com/foolin/goview"
	"github.com/foolin/goview/supports/gorice"
	"github.com/labstack/echo"
	"html/template"
	"io"
)

type TemplateRenderer struct {
	*goview.ViewEngine
}

func NewGoViewRenderer() *TemplateRenderer {
	box := rice.MustFindBox("../../web/templates")
	gv := gorice.NewWithConfig(box, goview.Config{
		Root:         "views",
		Extension:    ".html",
		Master:       "layouts/master",
		Partials:     []string{"partials/navbar"},
		Funcs:        make(template.FuncMap),
		DisableCache: false,
		Delims:       goview.Delims{Left: "{{", Right: "}}"},
	})
	return &TemplateRenderer{ViewEngine: gv}

}

func (t *TemplateRenderer) Render(w io.Writer, name string, data interface{}, c echo.Context) error {
	return t.ViewEngine.RenderWriter(w, name, data)
}
