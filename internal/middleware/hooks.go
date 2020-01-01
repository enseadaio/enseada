package middleware

import (
	"context"
	"encoding/base64"
	"fmt"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/twitchtv/twirp"
	"strings"
)

func AuthTwirpHooks(logger echo.Logger, s *auth.Store, op fosite.OAuth2Provider) *twirp.ServerHooks {
	h := &twirp.ServerHooks{}
	h.RequestRouted = func(ctx context.Context) (context.Context, error) {
		a, ok := AuthStrategy(ctx)
		if !ok {
			return ctx, nil
		}

		logger.Info("authenticating Twirp request")
		switch a.Prefix {
		case "basic":
			d, err := base64.StdEncoding.DecodeString(a.Content)
			if err != nil {
				return ctx, err
			}
			up := strings.Split(string(d), ":")
			if len(up) != 2 {
				logger.Errorf("authentication failed: %s", d)
				return ctx, fmt.Errorf("not a valid Basic auth value: %s", d)
			}

			err = s.Authenticate(ctx, up[0], up[1])
			if err != nil {
				logger.Errorf("authentication failed: %s", err.Error())
				return ctx, fmt.Errorf("authentication failed: %s", err.Error())
			}

			u, err := s.FindUserByUsername(ctx, up[0])
			if err != nil {
				logger.Error(err)
				return ctx, err
			}

			g := guid.NewWithRev(couch.UsersDB, u.ID, couch.KindUser, u.Rev)
			logger.Infof("successfully authenticated user %s", g.String())
			return SetCurrentUserGUID(ctx, g), nil
		case "bearer":
			ss := auth.NewSession(nil)
			tt, ar, err := op.IntrospectToken(ctx, a.Content, fosite.AccessToken, ss)
			if err != nil {
				logger.Error(err)
				return ctx, err
			}
			logger.Infof("successfully validated token of type %s", tt)
			id := ar.GetSession().GetSubject()
			u, err := s.GetUser(ctx, id)
			if err != nil {
				logger.Error(err)
				return ctx, err
			}

			g := guid.NewWithRev(couch.UsersDB, u.ID, couch.KindUser, u.Rev)
			logger.Infof("successfully authenticated user %s", g.String())
			return SetCurrentUserGUID(ctx, g), nil
		default:
			logger.Warnf("unknown authentication strategy: %s", a.Prefix)
			return ctx, nil
		}
	}

	return h
}
