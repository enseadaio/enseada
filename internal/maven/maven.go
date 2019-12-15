package maven

import (
	"github.com/chartmuseum/storage"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Maven struct {
	Logger  echo.Logger
	Data    *kivik.Client
	Storage storage.Backend
}
