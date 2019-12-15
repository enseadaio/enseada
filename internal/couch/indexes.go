package couch

import (
	"context"
	"fmt"
	"github.com/go-kivik/kivik"
	"github.com/labstack/gommon/log"
)

func InitIndex(ctx context.Context, client *kivik.Client, dbName string, name string, idx map[string]interface{}) error {
	db := client.DB(ctx, dbName)
	log.Infof("initializing index %s on db %s", name, dbName)
	return db.CreateIndex(ctx, fmt.Sprintf("%s_idx", dbName), name, idx)
}
