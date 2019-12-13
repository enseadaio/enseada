package main

import (
	"context"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/enseadaio/enseada/pkg/maven"
	"github.com/enseadaio/enseada/pkg/repo"
	"github.com/enseadaio/enseada/pkg/server"
	"github.com/enseadaio/enseada/pkg/storage"
	"github.com/joho/godotenv"
	"github.com/labstack/gommon/log"
	"github.com/spf13/viper"
	"os"
	"strings"
	"time"
)

func init() {
	if os.Getenv("ENSEADA_ENV") != "production" {
		err := godotenv.Load()
		if err != nil {
			log.Fatal("HTTPError loading .env file")
		}
	}

	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
}

func main() {
	provider := viper.GetString("storage.provider")
	localDir := viper.GetString("storage.dir")
	store, err := storage.Init(provider, storage.LocalDir(localDir))
	if err != nil {
		log.Fatal(err)
	}

	url := viper.GetString("couchdb.url")
	user := viper.GetString("couchdb.user")
	pwd := viper.GetString("couchdb.password")

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*60)
	defer cancel()

	db, err := couch.Init(ctx, url, user, pwd)
	if err != nil {
		log.Fatal(err)
	}

	srv := server.Create()

	r := &repo.Service{
		Logger: srv.Logger,
		Data:   db,
	}

	mvn := &maven.Maven{
		Logger:  srv.Logger,
		Data:    db,
		Storage: store,
	}
	server.Init(srv, r, mvn)
	srv.Logger.Fatal(srv.Start(":9623"))
}
