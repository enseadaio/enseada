package couch

import (
	"context"
	"github.com/go-kivik/kivik"
	"github.com/labstack/gommon/log"
)

func InitDb(ctx context.Context, client *kivik.Client, name string) error {
	does, err := client.DBExists(ctx, name)
	if err != nil {
		return err
	}
	if !does {
		log.Infof("initializing database %s", name)
		return client.CreateDB(ctx, name)
	}
	log.Infof("database %s already exists", name)
	return nil
}
