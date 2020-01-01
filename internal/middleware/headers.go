package middleware

import (
	"net/http"
	"strings"
)

type AuthorizationStrategy struct {
	Prefix  string
	Content string
}

func WithAuthorizationStrategy(base http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		h := r.Header.Get("authorization")
		if h == "" {
			base.ServeHTTP(w, r)
			return
		}

		ctx := r.Context()
		slices := strings.Split(h, " ")
		if len(slices) == 2 {
			a := AuthorizationStrategy{
				Prefix:  strings.ToLower(slices[0]),
				Content: strings.TrimSpace(slices[1]),
			}
			ctx = SetAuthStrategy(ctx, a)
		}

		r = r.WithContext(ctx)
		base.ServeHTTP(w, r)
		return
	})
}
