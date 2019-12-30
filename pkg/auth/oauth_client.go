// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"errors"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/ory/fosite"
	"golang.org/x/crypto/bcrypt"
)

type OAuthClient struct {
	ID            string     `json:"_id,omitempty"`
	Rev           string     `json:"_rev,omitempty"`
	Kind          couch.Kind `json:"kind"`
	HashedSecret  []byte     `json:"hashed_secret,omitempty"`
	RedirectURIs  []string   `json:"redirect_uris"`
	GrantTypes    []string   `json:"grant_types"`
	ResponseTypes []string   `json:"response_types"`
	Scopes        []string   `json:"scopes"`
	Audiences     []string   `json:"audiences"`
	Public        bool       `json:"public"`
}

type OAuthClientOptions struct {
	RedirectURIs  []string
	GrantTypes    []string
	ResponseTypes []string
	Scopes        []string
	Audiences     []string
	Public        bool
}

func NewOAuthClient(id string, secret string, opts ...OAuthClientOption) (*OAuthClient, error) {
	if id == "" {
		return nil, errors.New("client ID cannot be empty")
	}

	options := &OAuthClientOptions{
		RedirectURIs:  []string{},
		GrantTypes:    []string{},
		ResponseTypes: []string{},
		Scopes:        []string{},
		Audiences:     []string{},
		Public:        false,
	}

	for _, opt := range opts {
		opt(options)
	}

	if !options.Public && secret == "" {
		return nil, errors.New("client secret cannot be empty for non-public clients")
	}

	var hashed []byte
	if !options.Public {
		h, err := bcrypt.GenerateFromPassword([]byte(secret), bcrypt.DefaultCost)
		if err != nil {
			return nil, err
		}
		hashed = h
	}

	if len(options.GrantTypes) == 0 {
		options.GrantTypes = fosite.Arguments{"authorization_code"}
	}

	if len(options.ResponseTypes) == 0 {
		options.ResponseTypes = fosite.Arguments{"code"}
	}

	return &OAuthClient{
		ID:            id,
		Kind:          couch.KindOAuthClient,
		HashedSecret:  hashed,
		RedirectURIs:  options.RedirectURIs,
		GrantTypes:    options.GrantTypes,
		ResponseTypes: options.ResponseTypes,
		Scopes:        options.Scopes,
		Audiences:     options.Audiences,
		Public:        options.Public,
	}, nil
}

// GetID returns the client ID.
func (c *OAuthClient) GetID() string {
	return c.ID
}

func (c *OAuthClient) GetRev() string {
	return c.Rev
}

func (c *OAuthClient) SetRev(rev string) {
	c.Rev = rev
}

// GetHashedSecret returns the hashed secret as it is stored in the store.
func (c *OAuthClient) GetHashedSecret() []byte {
	return c.HashedSecret
}

// GetRedirectURIs returns the client's allowed redirect URIs.
func (c *OAuthClient) GetRedirectURIs() []string {
	return c.RedirectURIs
}

// GetGrantTypes returns the client's allowed grant types.
func (c *OAuthClient) GetGrantTypes() fosite.Arguments {
	return c.GrantTypes
}

// GetResponseTypes returns the client's allowed response types.
func (c *OAuthClient) GetResponseTypes() fosite.Arguments {
	return c.ResponseTypes
}

// GetScopes returns the scopes this client is allowed to request.
func (c *OAuthClient) GetScopes() fosite.Arguments {
	return c.Scopes
}

// IsPublic returns true, if this client is marked as public.
func (c *OAuthClient) IsPublic() bool {
	return c.Public
}

// GetAudience returns the allowed audience(s) for this client.
func (c *OAuthClient) GetAudience() fosite.Arguments {
	return c.Audiences
}
