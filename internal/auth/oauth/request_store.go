package oauth

import (
	"context"
	"errors"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"
)

type AuthRequestStore struct {
	data *kivik.Client
}

func NewRequestStore(db *kivik.Client) (*AuthRequestStore, error) {
	return &AuthRequestStore{data: db}, nil
}

func (t *AuthRequestStore) CreateAuthorizeCodeSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &Request{}
	req.Merge(request)
	return t.store(ctx, &RequestWrapper{
		Kind: couch.KindOAuthAuthorizeCode,
		Sig:  signature,
		Req:  req,
	})
}

func (t *AuthRequestStore) GetAuthorizeCodeSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
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

func (t *AuthRequestStore) InvalidateAuthorizeCodeSession(ctx context.Context, signature string) error {
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

func (t *AuthRequestStore) CreateAccessTokenSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &Request{}
	req.Merge(request)
	return t.store(ctx, &RequestWrapper{
		Kind: couch.KindOAuthAccessToken,
		Sig:  signature,
		Req:  req,
	})
}

func (t *AuthRequestStore) GetAccessTokenSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
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

func (t *AuthRequestStore) DeleteAccessTokenSession(ctx context.Context, signature string) error {
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

func (t *AuthRequestStore) CreateRefreshTokenSession(ctx context.Context, signature string, request fosite.Requester) error {
	req := &Request{}
	req.Merge(request)
	return t.store(ctx, &RequestWrapper{
		Kind: couch.KindOAuthRefreshToken,
		Sig:  signature,
		Req:  req,
	})
}

func (t *AuthRequestStore) GetRefreshTokenSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
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

func (t *AuthRequestStore) DeleteRefreshTokenSession(ctx context.Context, signature string) error {
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

func (t *AuthRequestStore) RevokeAccessToken(ctx context.Context, requestID string) error {
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

func (t *AuthRequestStore) RevokeRefreshToken(ctx context.Context, requestID string) error {
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

func (t *AuthRequestStore) store(ctx context.Context, token *RequestWrapper) error {
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

func (t *AuthRequestStore) get(ctx context.Context, id string) (*RequestWrapper, error) {
	db := t.data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)
	var token RequestWrapper
	if err := row.ScanDoc(&token); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, fosite.ErrNotFound
		}
		return nil, err
	}
	return &token, nil
}

func (t *AuthRequestStore) find(ctx context.Context, query interface{}) ([]*RequestWrapper, error) {
	var tokens []*RequestWrapper

	db := t.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, query)
	if err != nil {
		return tokens, err
	}

	for rows.Next() {
		var token RequestWrapper
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

func (t *AuthRequestStore) findOne(ctx context.Context, query interface{}) (*RequestWrapper, error) {
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

func (t *AuthRequestStore) delete(ctx context.Context, id string, rev string) error {
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
