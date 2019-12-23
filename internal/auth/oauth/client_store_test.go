package oauth

import (
	"context"
	"github.com/enseadaio/enseada/internal/auth/oidc"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/go-kivik/kivikmock"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestClientStore_GetByID(t *testing.T) {
	client, err := NewClient("test", "test")
	assert.NoError(t, err)
	client.Rev = "1"

	data, mock := kivikmock.NewT(t)
	db := mock.NewDB()
	mock.ExpectDBExists().WithName(couch.OAuthDBName).WillReturn(false)
	mock.ExpectCreateDB().WithName(couch.OAuthDBName)
	mock.ExpectDB().WithName(couch.OAuthDBName).WillReturn(db)
	db.ExpectCreateIndex().WithName("kind_index").WithIndex(map[string]interface{}{
		"fields": []string{"kind"},
	})

	mock.ExpectDB().WithName(couch.OAuthDBName).WillReturn(db)
	db.ExpectGet().WithDocID(client.ID).WillReturn(kivikmock.DocumentT(t, map[string]interface{}{
		"_id":            client.ID,
		"_rev":           client.Rev,
		"kind":           string(couch.KindOAuthClient),
		"hashed_secret":  client.HashedSecret,
		"redirect_uris":  client.RedirectURIs,
		"grant_types":    client.GrantTypes,
		"response_types": client.ResponseTypes,
		"scopes":         client.Scopes,
		"audiences":      client.Audiences,
		"public":         client.Public,
	}))
	store, err := NewClientStore(data)
	assert.NoError(t, err)

	got, err := store.GetClient(context.Background(), client.ID)
	assert.NoError(t, err)
	assert.Equal(t, client, got)
}

func TestClientStore_RegisterClient(t *testing.T) {
	client, err := NewClient("test", "test")
	assert.NoError(t, err)

	data, mock := kivikmock.NewT(t)
	db := mock.NewDB()
	mock.ExpectDB().WithName(couch.OAuthDBName).WillReturn(db)
	db.ExpectPut().WithDocID(client.ID).WillReturn("1")
	store := ClientStore{data: data}
	err = store.RegisterClient(context.Background(), client)
	assert.NoError(t, err)
	assert.Equal(t, "1", client.Rev)

	oid, err := oidc.NewOpenIDConnectClient("test", "test")
	assert.NoError(t, err)

	err := store.RegisterClient(context.Background(), oid)
}
