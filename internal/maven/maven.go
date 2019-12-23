package maven

import (
	"github.com/chartmuseum/storage"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Maven struct {
	logger  echo.Logger
	client  *kivik.Client
	storage storage.Backend
	dbname  string
}

const dbname = "maven2"

func New(client *kivik.Client, storage storage.Backend, logger echo.Logger) (*Maven, error) {
	return &Maven{
		logger:  logger,
		client:  client,
		storage: storage,
		dbname:  dbname,
	}, nil
}
