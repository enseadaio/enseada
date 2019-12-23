package couch

import (
	"context"
	"github.com/go-kivik/kivik"
	"github.com/pkg/errors"
)

const (
	MavenDB = "maven2"
	OAuthDB = "oauth"
	UsersDB = "users"
)

func Migrate(ctx context.Context, client *kivik.Client) error {
	if err := maven(ctx, client); err != nil {
		e := client.DestroyDB(ctx, MavenDB)
		if e != nil {
			return errors.Wrap(err, e.Error())
		}
		return err
	}

	if err := oauth(ctx, client); err != nil {
		e := client.DestroyDB(ctx, OAuthDB)
		if e != nil {
			return errors.Wrap(err, e.Error())
		}
		return err
	}

	if err := users(ctx, client); err != nil {
		e := client.DestroyDB(ctx, UsersDB)
		if e != nil {
			return errors.Wrap(err, e.Error())
		}
		return err
	}

	return nil
}

func maven(ctx context.Context, client *kivik.Client) error {

	if err := InitDb(ctx, client, MavenDB); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, MavenDB, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, MavenDB, "file_index", map[string]interface{}{
		"fields": []string{"files"},
	}); err != nil {
		return err
	}

	return nil
}

func oauth(ctx context.Context, client *kivik.Client) error {
	if err := InitDb(ctx, client, OAuthDB); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "kind_index", map[string]interface{}{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "oauth_reqs_index", Query{
		"fields": []string{"req.id"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "oauth_sigs_index", Query{
		"fields": []string{"sig"},
	}); err != nil {
		return err
	}

	if err := InitIndex(ctx, client, OAuthDB, "openid_reqs_index", Query{
		"fields": []string{"auth_code"},
	}); err != nil {
		return err
	}

	return nil
}

func users(ctx context.Context, client *kivik.Client) error {
	if err := InitDb(ctx, client, UsersDB); err != nil {
		return err
	}

	return nil
}
