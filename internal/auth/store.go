package auth

import (
	"context"
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"github.com/enseadaio/enseada/internal/auth/oidc"
	"github.com/enseadaio/enseada/internal/auth/pkce"
	"github.com/enseadaio/enseada/internal/users"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Store struct {
	data *kivik.Client
	*oauth.ClientStore
	*oauth.AuthRequestStore
	*oidc.SessionStore
	*pkce.RequestStore
	usvc *users.UserSvc
}

func NewStore(db *kivik.Client, logger echo.Logger) (*Store, error) {
	cs, err := oauth.NewClientStore(db, logger)
	if err != nil {
		return nil, err
	}

	ts, err := oauth.NewRequestStore(db)
	if err != nil {
		return nil, err
	}

	os, err := oidc.NewStore(db)
	if err != nil {
		return nil, err
	}

	pk, err := pkce.NewRequestStore(db)
	if err != nil {
		return nil, err
	}

	return &Store{
		data:             db,
		ClientStore:      cs,
		AuthRequestStore: ts,
		SessionStore:     os,
		RequestStore:     pk,
	}, nil
}

func (s *Store) Authenticate(ctx context.Context, username string, password string) error {
	_, err := s.usvc.Authenticate(ctx, username, password)
	return err
}
