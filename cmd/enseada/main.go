package main

import (
	"context"
	"fmt"
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

	viper.SetDefault("log.level", "info")
	viper.SetDefault("port", "9623")
	viper.SetDefault("storage.provider", "local")
	viper.SetDefault("storage.dir", "uploads")

	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
}

func main() {
	logLvl := getLogLvl(viper.GetString("log.level"))
	log.SetLevel(logLvl)

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

	srv := server.Create(logLvl)

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

	port := viper.GetString("port")
	srv.Logger.Fatal(srv.Start(fmt.Sprintf(":%s", port)))
}

func getLogLvl(lvl string) log.Lvl {
	switch strings.ToUpper(lvl) {
	case "DEBUG":
		return log.DEBUG
	case "INFO":
		return log.INFO
	case "WARN":
		return log.WARN
	case "ERROR":
		return log.ERROR
	case "OFF":
		return log.OFF
	default:
		return log.INFO
	}
}
