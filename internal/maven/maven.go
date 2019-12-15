package maven

import (
	"context"
	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"time"
)

type Maven struct {
	logger  echo.Logger
	client  *kivik.Client
	storage storage.Backend
	dbname  string
}

const dbname = "maven2"

func New(client *kivik.Client, storage storage.Backend, logger echo.Logger) (*Maven, error) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second*30)
	defer cancel()

	err := couch.InitDb(ctx, client, dbname)
	if err != nil {
		return nil, err
	}

	err = couch.InitIndex(ctx, client, dbname, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	})
	if err != nil {
		return nil, err
	}

	err = couch.InitIndex(ctx, client, dbname, "file_index", map[string]interface{}{
		"fields": []string{"files"},
	})
	if err != nil {
		return nil, err
	}

	return &Maven{
		logger:  logger,
		client:  client,
		storage: storage,
		dbname:  dbname,
	}, nil
}
