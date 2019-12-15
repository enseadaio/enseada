package server

import (
	rice "github.com/GeertJohan/go.rice"
	"github.com/labstack/echo"
	"net/http"
	"time"
)

func mountUI(e *echo.Echo) {
	staticHandler := http.FileServer(rice.MustFindBox("../../web/static").HTTPBox())
	e.GET("/static/*", echo.WrapHandler(http.StripPrefix("/static/", staticHandler)))

	assetHandler := http.FileServer(rice.MustFindBox("../../web/assets").HTTPBox())
	e.GET("/assets/*", echo.WrapHandler(http.StripPrefix("/assets/", assetHandler)))

	u := e.Group("/ui")

	u.GET("", home)
	u.GET("/repositories", repos())
}

func home(c echo.Context) error {
	return renderPage(c, "index", echo.Map{
		"Date": time.Now().String(),
	})
}

func repos() echo.HandlerFunc {
	return func(c echo.Context) error {
		return renderPage(c, "repos", echo.Map{})
	}
}

func renderPage(c echo.Context, name string, data interface{}) error {
	pusher, ok := c.Response().Writer.(http.Pusher)
	if ok {
		if err := pusher.Push("/static/main.css", nil); err != nil {
			return err
		}
		if err := pusher.Push("/static/app.js", nil); err != nil {
			return err
		}
	}
	return c.Render(http.StatusOK, name, data)
}
