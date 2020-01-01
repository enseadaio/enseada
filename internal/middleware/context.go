package middleware

import (
	"context"

	"github.com/ory/fosite"

	"github.com/enseadaio/enseada/internal/guid"
)

type contextKey int

const (
	currentUserGUID contextKey = 1 + iota
	authStrategy
	scopes
)

func CurrentUserGUID(ctx context.Context) (guid.GUID, bool) {
	g, ok := ctx.Value(currentUserGUID).(guid.GUID)
	return g, ok
}

func WithCurrentUserGUID(ctx context.Context, g guid.GUID) context.Context {
	return context.WithValue(ctx, currentUserGUID, g)
}

func Scopes(ctx context.Context) (fosite.Arguments, bool) {
	s, ok := ctx.Value(scopes).(fosite.Arguments)
	return s, ok
}

func WithScopes(ctx context.Context, s fosite.Arguments) context.Context {
	return context.WithValue(ctx, scopes, s)
}
