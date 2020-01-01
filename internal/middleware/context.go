package middleware

import (
	"context"
	"github.com/enseadaio/enseada/internal/guid"
)

type contextKey int

const (
	currentUserGUID contextKey = 1 + iota
	authStrategy
)

func CurrentUserGUID(ctx context.Context) (guid.GUID, bool) {
	g, ok := ctx.Value(currentUserGUID).(guid.GUID)
	return g, ok
}

func SetCurrentUserGUID(ctx context.Context, g guid.GUID) context.Context {
	return context.WithValue(ctx, currentUserGUID, g)
}

func AuthStrategy(ctx context.Context) (AuthorizationStrategy, bool) {
	a, ok := ctx.Value(authStrategy).(AuthorizationStrategy)
	return a, ok
}

func SetAuthStrategy(ctx context.Context, a AuthorizationStrategy) context.Context {
	return context.WithValue(ctx, authStrategy, a)
}
