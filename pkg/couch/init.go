package couch

import (
	"context"
	"github.com/go-kivik/couchdb"
	"github.com/go-kivik/kivik"
)

func Init(ctx context.Context, url string, user string, pwd string) (*kivik.Client, error) {
	client, err := kivik.New("couch", url)
	if err != nil {
		return nil, err
	}

	err = client.Authenticate(context.Background(), couchdb.BasicAuth(user, pwd))
	if err != nil {
		return nil, err
	}

	if err := initDbs(ctx, client); err != nil {
		return nil, err
	}

	if err := initViews(ctx, client); err != nil {
		return nil, err
	}

	if err := initIndexes(ctx, client); err != nil {
		return nil, err
	}

	return client, nil
}
