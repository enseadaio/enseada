package main

import (
	"context"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/joho/godotenv"
	"github.com/spf13/viper"
	"log"
	"strings"
)

func init() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal("HTTPError loading .env file")
	}

	viper.AutomaticEnv()
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
}

func main() {
	url := viper.GetString("couchdb.url")
	user := viper.GetString("couchdb.user")
	pwd := viper.GetString("couchdb.password")

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	c, err := couch.Init(ctx, url, user, pwd)
	if err != nil {
		log.Fatal(err)
	}

	db := c.DB(ctx, "repositories")
	rows, err := db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"files": map[string]interface{}{
				"$elemMatch": map[string]interface{}{
					"$eq": "maven2/com/matteojoliveau/test-enseada/test-enseada/maven-metadata.xml",
				},
			},
		},
	})

	if err != nil {
		log.Fatal(err)
	}

	for rows.Next() {
		log.Printf("By file: %s", rows.Key())
	}
	log.Printf("Total Files: %d", rows.TotalRows())

	rows, err = db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"kind": "repository",
		},
	})

	if err != nil {
		log.Fatal(err)
	}

	for rows.Next() {
		log.Printf("Repo: %s", rows.Key())
	}
	log.Printf("Total Repos: %d", rows.TotalRows())
}
