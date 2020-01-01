package middleware

import (
	"context"

	"github.com/ory/fosite"
)

type contextKey int

const (
	currentUserID contextKey = 1 + iota
	authStrategy
	scopes
)

func CurrentUserID(ctx context.Context) (string, bool) {
	g, ok := ctx.Value(currentUserID).(string)
	return g, ok
}

func WithCurrentUserID(ctx context.Context, g string) context.Context {
	return context.WithValue(ctx, currentUserID, g)
}

func Scopes(ctx context.Context) (fosite.Arguments, bool) {
	s, ok := ctx.Value(scopes).(fosite.Arguments)
	return s, ok
}

func WithScopes(ctx context.Context, s fosite.Arguments) context.Context {
	return context.WithValue(ctx, scopes, s)
}
