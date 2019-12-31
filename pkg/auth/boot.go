package auth

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	rice "github.com/GeertJohan/go.rice"
	"github.com/casbin/casbin/v2"
	"github.com/casbin/casbin/v2/model"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/ory/fosite/compose"
	goauth "golang.org/x/oauth2"
)

type Components struct {
	Store    *Store
	Enforcer *casbin.Enforcer
	Watcher  *CasbinWatcher
	Provider fosite.OAuth2Provider
	Client   *goauth.Config
}

func Boot(ctx context.Context, data *kivik.Client, logger echo.Logger, skb []byte, ph string, clientSecret string) (*Components, error) {
	if err := couch.Transact(ctx, data, migrateAclDb, couch.AclDB); err != nil {
		return nil, err
	}

	if err := couch.Transact(ctx, data, migrateOAuthDb, couch.OAuthDB); err != nil {
		return nil, err
	}

	if err := couch.Transact(ctx, data, migrateUsersDb, couch.UsersDB); err != nil {
		return nil, err
	}

	s := createStore(data, logger)

	e, w, err := createCasbin(data, logger)
	if err != nil {
		return nil, err
	}

	key, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, err
	}

	o := compose.ComposeAllEnabled(
		&compose.Config{},
		s,
		skb,
		key,
	)

	redirect := ph + "/ui/callback"
	oc := &goauth.Config{
		ClientID:     "enseada",
		ClientSecret: clientSecret,
		Endpoint: goauth.Endpoint{
			AuthURL:   ph + "/oauth/authorize",
			TokenURL:  ph + "/oauth/token",
			AuthStyle: goauth.AuthStyleAutoDetect,
		},
		RedirectURL: redirect,
		Scopes:      []string{"openid", "profile"},
	}

	ec, err := s.GetClient(ctx, "enseada")
	if err != nil {
		return nil, err
	}
	if ec == nil {
		defaultClient, err := NewOAuthClient("enseada", clientSecret,
			OAuthGrantTypes("authorization_code", "implicit", "refresh_token", "password", "client_credentials"),
			OAuthResponseTypes("code", "id_token", "token id_token", "code id_token", "code token", "code token id_token"),
			OAuthScopes("openid", "profile"),
			OAuthRedirectURIs(redirect),
		)
		if err != nil {
			return nil, err
		}

		if err := s.RegisterClient(ctx, defaultClient); err != nil {
			return nil, err
		}
	}

	fr, err := s.FindByUsername(ctx, "root")
	if err != nil {
		return nil, err
	}

	if fr == nil {
		root := RootUser("root")
		if err := s.Save(ctx, root); err != nil {
			return nil, err
		}
	}

	return &Components{
		Store:    s,
		Enforcer: e,
		Watcher:  w,
		Provider: o,
		Client:   oc,
	}, nil
}

func createStore(data *kivik.Client, logger echo.Logger) *Store {
	oAuthClientStore := NewOAuthClientStore(data, logger)
	oAuthRequestStore := NewOAuthRequestStore(data, logger)
	oidcSessionStore := NewOIDCSessionStore(data, logger)
	pkceRequestStore := NewPKCERequestStore(data, logger)
	userStore := NewUserStore(data, logger)
	return NewStore(data, logger, oAuthClientStore, oAuthRequestStore, oidcSessionStore, pkceRequestStore, userStore)
}

func createCasbin(data *kivik.Client, logger echo.Logger) (*casbin.Enforcer, *CasbinWatcher, error) {
	box := rice.MustFindBox("../../conf/")
	model, err := model.NewModelFromString(box.MustString("casbin_model.conf"))
	if err != nil {
		return nil, nil, err
	}

	adapter, err := NewCasbinAdapter(data, logger)
	if err != nil {
		return nil, nil, err
	}

	watcher := NewCasbinWatcher(data, logger)

	e, err := casbin.NewEnforcer(model, adapter)
	if err != nil {
		return nil, nil, err
	}

	e.EnableLog(false)
	e.EnableAutoSave(true)

	err = e.SetWatcher(watcher)
	if err != nil {
		return nil, nil, err
	}

	return e, watcher, nil
}

func migrateAclDb(ctx context.Context, client *kivik.Client) error {
	if err := couch.InitDb(ctx, client, couch.AclDB); err != nil {
		return err
	}

	return nil
}

func migrateOAuthDb(ctx context.Context, client *kivik.Client) error {
	if err := couch.InitDb(ctx, client, couch.OAuthDB); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, client, couch.OAuthDB, "kind_index", couch.Query{
		"fields": []string{"kind"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, client, couch.OAuthDB, "oauth_reqs_index", couch.Query{
		"fields": []string{"req.id"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, client, couch.OAuthDB, "oauth_sigs_index", couch.Query{
		"fields": []string{"sig"},
	}); err != nil {
		return err
	}

	if err := couch.InitIndex(ctx, client, couch.OAuthDB, "openid_reqs_index", couch.Query{
		"fields": []string{"auth_code"},
	}); err != nil {
		return err
	}

	return nil
}

func migrateUsersDb(ctx context.Context, client *kivik.Client) error {
	if err := couch.InitDb(ctx, client, couch.UsersDB); err != nil {
		return err
	}

	return nil
}
