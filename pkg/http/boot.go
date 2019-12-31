package http

import (
	"context"
	"github.com/enseadaio/enseada/pkg/auth"
	"github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
	goauth "golang.org/x/oauth2"
	"time"
)

func Boot(_ context.Context, lvl log.Lvl, op fosite.OAuth2Provider, oc *goauth.Config, store *auth.Store, skb []byte) (*echo.Echo, error) {
	e := createEchoServer(lvl)

	s := sessionStore(skb)
	sm := session.Sessions("enseada-session", s)
	mountHealthCheck(e)
	mountUI(e, oc, sm)
	mountAuth(e, op, store, sm)
	return e, nil
}

func sessionStore(skb []byte) session.Store {
	exp := (time.Hour * 720).Seconds()
	s := session.NewCookieStore(skb)
	s.Options(session.Options{
		MaxAge:   int(exp),
		HttpOnly: true,
	})
	return s
}
