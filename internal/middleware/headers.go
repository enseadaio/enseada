// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package middleware

import (
	"encoding/base64"
	"fmt"
	"net/http"
	"strings"

	"github.com/enseadaio/enseada/internal/scope"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/ctxutils"

	"github.com/enseadaio/enseada/internal/auth"
	"github.com/ory/fosite"
)

func AuthorizationHeader(logger log.Logger, s *auth.Store, op fosite.OAuth2Provider) func(http.Handler) http.Handler {
	return func(base http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if strings.HasPrefix(r.RequestURI, "/oauth") {
				base.ServeHTTP(w, r)
				return
			}

			h := r.Header.Get("authorization")
			if h == "" {
				base.ServeHTTP(w, r)
				return
			}

			ctx := r.Context()
			slices := strings.Split(h, " ")
			if len(slices) != 2 {
				base.ServeHTTP(w, r)
				return
			}

			prefix := strings.ToLower(strings.TrimSpace(slices[0]))
			content := strings.TrimSpace(slices[1])

			var token string
			switch prefix {
			case "basic":
				d, err := base64.StdEncoding.DecodeString(content)
				if err != nil {
					http.Error(w, err.Error(), http.StatusBadRequest)
					return
				}
				up := strings.Split(string(d), ":")
				if len(up) != 2 {
					logger.Errorf("authentication failed: %s", d)
					http.Error(w, fmt.Sprintf("not a valid Basic auth value: %s", d), http.StatusBadRequest)
					return
				}
				if up[0] != "x-oauth-token" {
					http.Error(w,
						"invalid username. HTTP Basic auth requires special user 'x-oauth-token' with a valid OAuth 2.0 token as password",
						http.StatusUnauthorized)
					return
				}
				token = up[1]
			case "bearer":
				token = content
			default:
				logger.Warnf("unknown authentication strategy: %s", prefix)
				base.ServeHTTP(w, r)
				return
			}

			ss := auth.NewSession(nil)
			tt, ar, err := op.IntrospectToken(ctx, token, fosite.AccessToken, ss)
			if err != nil {
				logger.Error(err)
				http.Error(w, err.Error(), http.StatusUnauthorized)
				return
			}
			logger.Infof("successfully validated token of type %s", tt)
			id := ar.GetSession().GetSubject()
			u, err := s.GetUser(ctx, id)
			if err != nil {
				logger.Error(err)
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}

			logger.Infof("successfully authenticated user %s", u.Username)
			ctx = ctxutils.WithCurrentUserID(ctx, u.Username)

			scs := make([]string, 0)
			for _, s := range scope.All {
				if fosite.WildcardScopeStrategy(ar.GetGrantedScopes(), s) {
					scs = append(scs, s)
				}
			}
			ctx = ctxutils.WithScopes(ctx, scs)

			r = r.WithContext(ctx)
			base.ServeHTTP(w, r)
			return
		})
	}
}
