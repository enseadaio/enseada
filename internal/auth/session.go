package auth

import (
	"time"

	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
	"github.com/ory/fosite/token/jwt"
)

func NewSession(u *User) fosite.Session {
	if u == nil {
		return &openid.DefaultSession{
			Claims: &jwt.IDTokenClaims{
				Issuer:      "enseada",
				Subject:     "",
				Audience:    []string{"enseada"},
				Nonce:       "",
				ExpiresAt:   time.Now().Add(time.Hour * 6),
				IssuedAt:    time.Now(),
				RequestedAt: time.Now(),
				AuthTime:    time.Now(),
			},
			Username: "",
			Subject:  "",
		}
	}

	return &openid.DefaultSession{
		Claims: &jwt.IDTokenClaims{
			Issuer:      "enseada",
			Subject:     u.ID,
			Audience:    []string{"enseada"},
			Nonce:       "",
			ExpiresAt:   time.Now().Add(time.Hour * 6),
			IssuedAt:    time.Now(),
			RequestedAt: time.Now(),
			AuthTime:    time.Now(),
			Extra: echo.Map{
				"username": u.Username,
			},
		},
		Username: u.Username,
		Subject:  u.ID,
	}
}
