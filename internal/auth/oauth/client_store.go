package oauth

import (
	"context"
	"errors"
	"fmt"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
)

type ClientStore struct {
	logger echo.Logger
	data   *kivik.Client
}

func NewClientStore(db *kivik.Client, logger echo.Logger) (*ClientStore, error) {
	return &ClientStore{data: db, logger: logger}, nil
}

func (c *ClientStore) GetClient(ctx context.Context, id string) (fosite.Client, error) {
	log.Debugf("Getting client with id %s", id)
	db := c.data.DB(ctx, couch.OAuthDB)
	row := db.Get(ctx, id)

	var client Client
	if err := row.ScanDoc(&client); err != nil {
		log.Error(err)
		return nil, err
	}

	return &client, nil
}

func (c *ClientStore) RegisterClient(ctx context.Context, client fosite.Client) error {
	cl, ok := client.(couch.Storable)
	if !ok {
		return errors.New(fmt.Sprintf("client %s does not implement couch.Storable", client.GetID()))
	}
	db := c.data.DB(ctx, couch.OAuthDB)
	rev, err := db.Put(ctx, cl.GetID(), client)
	if err != nil {
		return err
	}

	cl.SetRev(rev)
	return nil
}

func (c *ClientStore) InitDefaultClient(ctx context.Context, publicHost string, secret string) error {
	db := c.data.DB(ctx, couch.OAuthDB)
	_, _, err := db.GetMeta(ctx, "enseada")
	if err == nil {
		return nil
	}
	if kivik.StatusCode(err) != kivik.StatusNotFound {
		return err
	}

	client, err := NewClient("enseada", secret,
		RedirectURIs(publicHost+"/ui/callback"),
		Scopes("openid"),
	)
	if err != nil {
		return err
	}
	err = c.RegisterClient(ctx, client)
	if err != nil {
		return err
	}

	c.logger.Infof("Created default OAuthServer client. client_id: %s client_secret: %s", "enseada", secret)
	return nil
}
