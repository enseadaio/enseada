package couch

import (
	"context"
	"github.com/go-kivik/couchdb"
	"github.com/go-kivik/kivik"
)

func NewClient(ctx context.Context, url string, user string, pwd string) (*kivik.Client, error) {
	client, err := kivik.New("couch", url)
	if err != nil {
		return nil, err
	}

	err = client.Authenticate(context.Background(), couchdb.BasicAuth(user, pwd))
	return client, err
}
