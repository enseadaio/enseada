package http

import (
	"context"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	goauth "golang.org/x/oauth2"
)

func Boot(_ context.Context, lvl log.Lvl, oc *goauth.Config, skb []byte) (*echo.Echo, error) {
	e := createEchoServer(lvl)

	mountHealthCheck(e)
	mountUI(e, oc, middleware.Session(skb))
	return e, nil
}
