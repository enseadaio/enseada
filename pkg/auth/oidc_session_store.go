// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
)

type OIDCSessionStore struct {
	data   *kivik.Client
	logger echo.Logger
}

func NewOIDCSessionStore(data *kivik.Client, logger echo.Logger) *OIDCSessionStore {
	return &OIDCSessionStore{data: data, logger: logger}
}

func (s *OIDCSessionStore) CreateOpenIDConnectSession(ctx context.Context, authorizeCode string, requester fosite.Requester) error {
	req := &OAuthRequest{}
	req.Merge(requester)
	db := s.data.DB(ctx, couch.OAuthDB)
	_, _, err := db.CreateDoc(ctx, &OIDCSession{
		Kind:     couch.KindOpenIDSession,
		AuthCode: authorizeCode,
		Req:      req,
	})
	return err
}

func (s *OIDCSessionStore) GetOpenIDConnectSession(ctx context.Context, authorizeCode string, requester fosite.Requester) (fosite.Requester, error) {
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

	var session OIDCSession
	if rows.Next() {
		if err := rows.ScanDoc(&session); err != nil {
			return nil, err
		}
		requester.SetSession(session.Req.GetSession())
		return session.Req, nil
	}

	return nil, openid.ErrNoSessionFound
}

func (s *OIDCSessionStore) DeleteOpenIDConnectSession(ctx context.Context, authorizeCode string) error {
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

	var session OIDCSession
	if rows.Next() {
		if err := rows.ScanDoc(&session); err != nil {
			return err
		}
		_, err = db.Delete(ctx, session.ID, session.Rev)
		return err
	}
	return openid.ErrNoSessionFound
}
