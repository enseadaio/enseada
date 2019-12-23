package oidc

import (
	"context"
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
)

type SessionStore struct {
	data *kivik.Client
}

func NewStore(db *kivik.Client) (*SessionStore, error) {
	return &SessionStore{data: db}, nil
}

func (s *SessionStore) CreateOpenIDConnectSession(ctx context.Context, authorizeCode string, requester fosite.Requester) error {
	req := &oauth.Request{}
	req.Merge(requester)
	db := s.data.DB(ctx, couch.OAuthDB)
	_, _, err := db.CreateDoc(ctx, &Session{
		Kind:     couch.KindOpenIDSession,
		AuthCode: authorizeCode,
		Req:      req,
	})
	return err
}

func (s *SessionStore) GetOpenIDConnectSession(ctx context.Context, authorizeCode string, requester fosite.Requester) (fosite.Requester, error) {
	db := s.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, couch.Query{
		"selector": couch.Query{
			"kind":      couch.KindOpenIDSession,
			"auth_code": authorizeCode,
		},
	})
	if err != nil {
		return nil, err
	}

	var session Session
	if rows.Next() {
		if err := rows.ScanDoc(&session); err != nil {
			return nil, err
		}
		requester.SetSession(session.Req.GetSession())
		return session.Req, nil
	}

	return nil, openid.ErrNoSessionFound
}

func (s *SessionStore) DeleteOpenIDConnectSession(ctx context.Context, authorizeCode string) error {
	db := s.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, couch.Query{
		"selector": couch.Query{
			"kind":      couch.KindOpenIDSession,
			"auth_code": authorizeCode,
		},
	})
	if err != nil {
		return err
	}

	var session Session
	if rows.Next() {
		if err := rows.ScanDoc(&session); err != nil {
			return err
		}
		_, err = db.Delete(ctx, session.ID, session.Rev)
		return err
	}
	return openid.ErrNoSessionFound
}
