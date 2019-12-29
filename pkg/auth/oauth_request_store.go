package auth

import (
	"context"
	"errors"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
)

type OAuthRequestStore struct {
	data   *kivik.Client
	logger echo.Logger
}

func NewOAuthRequestStore(data *kivik.Client, logger echo.Logger) *OAuthRequestStore {
	return &OAuthRequestStore{data: data, logger: logger}
}

func (t *OAuthRequestStore) CreateAuthorizeCodeSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &OAuthRequest{}
	req.Merge(request)
	return t.store(ctx, &OAuthRequestWrapper{
		Kind: couch.KindOAuthAuthorizeCode,
		Sig:  signature,
		Req:  req,
	})
}

func (t *OAuthRequestStore) GetAuthorizeCodeSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthAuthorizeCode,
			"sig":  signature,
		},
	})
	if err != nil {
		return nil, err
	}

	if token.Revoked {
		return token.Req, fosite.ErrInvalidatedAuthorizeCode
	}

	session = token.Req.GetSession()
	return token.Req, nil
}

func (t *OAuthRequestStore) InvalidateAuthorizeCodeSession(ctx context.Context, signature string) error {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthAuthorizeCode,
			"sig":  signature,
		},
	})
	if err != nil {
		return err
	}

	return t.delete(ctx, token.ID, token.Rev)
}

func (t *OAuthRequestStore) CreateAccessTokenSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &OAuthRequest{}
	req.Merge(request)
	return t.store(ctx, &OAuthRequestWrapper{
		Kind: couch.KindOAuthAccessToken,
		Sig:  signature,
		Req:  req,
	})
}

func (t *OAuthRequestStore) GetAccessTokenSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthAccessToken,
			"sig":  signature,
		},
	})
	if err != nil {
		return nil, err
	}

	if token.Revoked {
		return nil, fosite.ErrInactiveToken
	}

	session = token.Req.GetSession()
	return token.Req, nil
}

func (t *OAuthRequestStore) DeleteAccessTokenSession(ctx context.Context, signature string) error {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthAccessToken,
			"sig":  signature,
		},
	})
	if err != nil {
		return err
	}

	return t.delete(ctx, token.ID, token.Rev)
}

func (t *OAuthRequestStore) CreateRefreshTokenSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &OAuthRequest{}
	req.Merge(request)
	return t.store(ctx, &OAuthRequestWrapper{
		Kind: couch.KindOAuthRefreshToken,
		Sig:  signature,
		Req:  req,
	})
}

func (t *OAuthRequestStore) GetRefreshTokenSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthRefreshToken,
			"sig":  signature,
		},
	})
	if err != nil {
		return nil, err
	}

	if token.Revoked {
		return nil, fosite.ErrInactiveToken
	}

	session = token.Req.GetSession()
	return token.Req, nil
}

func (t *OAuthRequestStore) DeleteRefreshTokenSession(ctx context.Context, signature string) error {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindOAuthRefreshToken,
			"sig":  signature,
		},
	})
	if err != nil {
		return err
	}

	return t.delete(ctx, token.ID, token.Rev)
}

func (t *OAuthRequestStore) RevokeAccessToken(ctx context.Context, requestID string) error {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"req.id": requestID,
			"kind":   couch.KindOAuthAccessToken,
		},
	})
	if err != nil {
		return err
	}

	token.Revoked = true
	return t.store(ctx, token)
}

func (t *OAuthRequestStore) RevokeRefreshToken(ctx context.Context, requestID string) error {
	token, err := t.findOne(ctx, couch.Query{
		"selector": couch.Query{
			"req.id": requestID,
			"kind":   couch.KindOAuthRefreshToken,
		},
	})
	if err != nil {
		return err
	}

	token.Revoked = true
	return t.store(ctx, token)
}

func (t *OAuthRequestStore) store(ctx context.Context, token *OAuthRequestWrapper) error {
	db := t.data.DB(ctx, couch.OAuthDB)
	if token.ID != "" && token.Rev == "" {
		_, rev, err := db.GetMeta(ctx, token.ID)
		if err != nil {
			if kivik.StatusCode(err) != kivik.StatusNotFound {
				return err
			}
		}

		token.Rev = rev
	}
	if token.ID == "" {
		id, rev, err := db.CreateDoc(ctx, token)
		if err != nil {
			return err
		}

		token.ID = id
		token.Rev = rev
		return nil
	} else {
		rev, err := db.Put(ctx, token.ID, token)
		if err != nil {
			return err
		}
		token.Rev = rev
		return nil
	}
}

func (t *OAuthRequestStore) get(ctx context.Context, id string) (*OAuthRequestWrapper, error) {
	db := t.data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)
	var token OAuthRequestWrapper
	if err := row.ScanDoc(&token); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, fosite.ErrNotFound
		}
		return nil, err
	}
	return &token, nil
}

func (t *OAuthRequestStore) find(ctx context.Context, query interface{}) ([]*OAuthRequestWrapper, error) {
	var tokens []*OAuthRequestWrapper

	db := t.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, query)
	if err != nil {
		return tokens, err
	}

	for rows.Next() {
		var token OAuthRequestWrapper
		if err := rows.ScanDoc(&token); err != nil {
			return tokens, err
		}
		tokens = append(tokens, &token)
	}

	if rows.Err() != nil {
		return tokens, rows.Err()
	}

	return tokens, nil
}

func (t *OAuthRequestStore) findOne(ctx context.Context, query interface{}) (*OAuthRequestWrapper, error) {
	tokens, err := t.find(ctx, query)
	if err != nil {
		return nil, err
	}

	if len(tokens) == 0 {
		return nil, fosite.ErrNotFound
	}

	if len(tokens) > 1 {
		return nil, errors.New("too many results")
	}

	return tokens[0], nil
}

func (t *OAuthRequestStore) delete(ctx context.Context, id string, rev string) error {
	db := t.data.DB(ctx, couch.OAuthDB)
	if rev == "" {
		_, r, err := db.GetMeta(ctx, id)
		if err != nil {
			if kivik.StatusCode(err) == kivik.StatusNotFound {
				return fosite.ErrNotFound
			}
			return err
		}
		rev = r
	}

	_, err := db.Delete(ctx, id, rev)
	return err
}
