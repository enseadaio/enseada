package boot

import (
	"context"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/go-kivik/kivik"
	"github.com/spf13/viper"
)

func dbClient(ctx context.Context, conf *viper.Viper) (*kivik.Client, error) {
	url := conf.GetString("couchdb.url")
	user := conf.GetString("couchdb.user")
	pwd := conf.GetString("couchdb.password")

	return couch.NewClient(ctx, url, user, pwd)
}
