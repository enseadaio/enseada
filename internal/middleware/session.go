package middleware

import (
	"time"

	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
)

func Session(skb []byte) echo.MiddlewareFunc {
	exp := (time.Hour * 720).Seconds()
	s := session.NewCookieStore(skb)
	s.Options(session.Options{
		MaxAge:   int(exp),
		HttpOnly: true,
	})
	return session.Sessions("enseada-session", s)
}