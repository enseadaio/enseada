// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"testing"

	"github.com/enseadaio/enseada/pkg/log/adapters"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/go-kivik/kivikmock"
	"github.com/stretchr/testify/assert"
)

func TestOAuthClientStore_GetByID(t *testing.T) {
	client, err := NewOAuthClient("test", "test")
	assert.NoError(t, err)
	client.Rev = "1"

	data, mock := kivikmock.NewT(t)
	db := mock.NewDB()
	mock.ExpectDB().WithName(couch.OAuthDB).WillReturn(db)
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
	l, err := adapters.NewZapLoggerAdapter(log.INFO)
	assert.NoError(t, err)

	store := NewOAuthClientStore(data, l)
	assert.NoError(t, err)

	got, err := store.GetClient(context.Background(), client.ID)
	assert.NoError(t, err)
	assert.Equal(t, client, got)
}
