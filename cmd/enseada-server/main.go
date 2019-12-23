package main

import (
	"context"
	"fmt"
	rice "github.com/GeertJohan/go.rice"
	"github.com/casbin/casbin/v2"
	"github.com/casbin/casbin/v2/model"
	"github.com/enseadaio/enseada/internal/auth/acl"
	"github.com/enseadaio/enseada/internal/users"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/enseadaio/enseada/pkg/storage"
	"github.com/joho/godotenv"
	"github.com/labstack/gommon/log"
	"github.com/spf13/viper"
	"os"
	"strings"
	"time"
)

func init() {
	if info, err := os.Stat("./.env"); err == nil && !info.IsDir() {
		err := godotenv.Load()
		if err != nil {
			log.Fatalf("Error loading .env file: %s", err.Error())
		}
	}

	viper.SetDefault("log.level", "info")
	viper.SetDefault("port", "9623")
	viper.SetDefault("storage.provider", "local")
	viper.SetDefault("storage.dir", "uploads")
	viper.SetDefault("root.password", "root")

	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
}

func main() {
	logLvl := getLogLvl(viper.GetString("log.level"))
	log.SetLevel(logLvl)

	provider := viper.GetString("storage.provider")
	localDir := viper.GetString("storage.dir")
	store, err := storage.NewBackend(provider, storage.LocalDir(localDir))
	exitOnErr(err)

	url := viper.GetString("couchdb.url")
	user := viper.GetString("couchdb.user")
	pwd := viper.GetString("couchdb.password")

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*60)
	defer cancel()

	db, err := couch.NewClient(ctx, url, user, pwd)
	exitOnErr(err)

	box := rice.MustFindBox("../../conf/")
	models, err := model.NewModelFromString(box.MustString("casbin_model.conf"))
	exitOnErr(err)

	casbinLog := log.New("casbin")
	casbinLog.SetLevel(logLvl)
	a, err := acl.NewAdapter(db, "casbin", casbinLog)
	exitOnErr(err)

	e, err := casbin.NewEnforcer(models, a)
	exitOnErr(err)

	userLog := log.New("users")
	rootPwd := viper.GetString("root.password")

	usvc := users.NewSvc(db, userLog)
	err = usvc.Save(ctx, users.Root(rootPwd))
	exitOnErr(err)

	publicHost := viper.GetString("public.host")
	sec := viper.GetString("default.oauth.client.secret")
	skb := viper.GetString("secret.key.base")
	srv, err := enseada.NewServer(db, store, e, usvc,
		enseada.ServerLogLevel(logLvl),
		enseada.ServerDefaultOAuthClientSecret(sec),
		enseada.ServerPublicHost(publicHost),
		enseada.ServerSecretKeyBase(skb),
	)
	exitOnErr(err)

	srv.Init()

	port := viper.GetString("port")
	sslVar := viper.GetString("ssl")
	ssl := sslVar != "" && sslVar != "false" && sslVar != "no"

	address := fmt.Sprintf(":%s", port)
	if ssl {
		cert := viper.GetString("ssl.cert.path")
		key := viper.GetString("ssl.key.path")
		err = srv.StartTLS(address, cert, key)
	} else {
		err = srv.Start(address)
	}
	srv.Logger.Fatal(err)
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

func exitOnErr(err error) {
	if err != nil {
		log.Fatal(err)
	}
}
