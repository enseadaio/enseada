package couch

import (
	"context"
	"fmt"
	"github.com/go-kivik/kivik"
	"github.com/labstack/gommon/log"
)

func initIndexes(ctx context.Context, client *kivik.Client) error {
	err := initIndex(ctx, client, "repositories", "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	})
	if err != nil {
		return err
	}

	err = initIndex(ctx, client, "repositories", "type_index", map[string]interface{}{
		"fields": []string{"type"},
	})
	if err != nil {
		return err
	}

	err = initIndex(ctx, client, "repositories", "file_index", map[string]interface{}{
		"fields": []string{"files"},
	})
	if err != nil {
		return err
	}

	return nil
}

func initIndex(ctx context.Context, client *kivik.Client, dbName string, name string, idx map[string]interface{}) error {
	db := client.DB(ctx, dbName)
	log.Infof("initializing index %s on db %s", name, dbName)
	return db.CreateIndex(ctx, fmt.Sprintf("%s_idx", dbName), name, idx)
}
