package pkce

import (
	"context"
	"errors"
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"
)

type RequestStore struct {
	data *kivik.Client
}

func NewRequestStore(db *kivik.Client) (*RequestStore, error) {
	return &RequestStore{data: db}, nil
}

func (r *RequestStore) CreatePKCERequestSession(ctx context.Context, signature string, requester fosite.Requester) error {
	req := &oauth.Request{}
	req.Merge(requester)
	db := r.data.DB(ctx, couch.OAuthDB)
	_, _, err := db.CreateDoc(ctx, &oauth.RequestWrapper{
		Kind: couch.KindPKCERequest,
		Sig:  signature,
		Req:  req,
	})
	return err
}

func (r *RequestStore) GetPKCERequestSession(ctx context.Context, signature string, session fosite.Session) (fosite.Requester, error) {
	db := r.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindPKCERequest,
			"sig":  signature,
		},
	})
	if err != nil {
		return nil, err
	}

	var request oauth.RequestWrapper
	if rows.Next() {
		if err := rows.ScanDoc(&request); err != nil {
			return nil, err
		}
		session = request.Req.GetSession()
		return request.Req, nil
	}

	return nil, errors.New("pkce request not found")
}

func (r *RequestStore) DeletePKCERequestSession(ctx context.Context, signature string) error {
	db := r.data.DB(ctx, couch.OAuthDB)
	rows, err := db.Find(ctx, couch.Query{
		"selector": couch.Query{
			"kind": couch.KindPKCERequest,
			"sig":  signature,
		},
	})
	if err != nil {
		return err
	}

	var request oauth.RequestWrapper
	if rows.Next() {
		if err := rows.ScanDoc(&request); err != nil {
			return err
		}
		_, err = db.Delete(ctx, request.ID, request.Rev)
		return err
	}
	return errors.New("pkce request not found")
}
