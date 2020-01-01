// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"gopkg.in/square/go-jose.v2"
)

type OIDCClient struct {
	*OAuthClient
	JSONWebKeysURI                string              `json:"jwks_uri"`
	JSONWebKeys                   *jose.JSONWebKeySet `json:"jwks"`
	TokenEndpointAuthMethod       string              `json:"token_endpoint_auth_method"`
	RequestURIs                   []string            `json:"request_uris"`
	RequestObjectSigningAlgorithm string              `json:"request_object_signing_alg"`
}

func NewOIDCClient(id string, secret string, opts ...OIDCClientOption) (*OIDCClient, error) {
	options := &OIDCClientOptions{
		OAuthClientOptions: []OAuthClientOption{},
		JwksURI:            "",
		Jwks:               nil,
		TokenAuthMethod:    "",
		RequestURIs:        nil,
		RequestAlgo:        "",
	}

	for _, opt := range opts {
		opt(options)
	}

	oc, err := NewOAuthClient(id, secret, options.OAuthClientOptions...)
	if err != nil {
		return nil, err
	}

	return &OIDCClient{
		OAuthClient:                   oc,
		JSONWebKeysURI:                options.JwksURI,
		JSONWebKeys:                   options.Jwks,
		TokenEndpointAuthMethod:       options.TokenAuthMethod,
		RequestURIs:                   options.RequestURIs,
		RequestObjectSigningAlgorithm: options.RequestAlgo,
	}, nil
}

// GetRequestURIs is an array of request_uri values that are pre-registered by the RP for use at the OP. Servers MAY
// cache the contents of the files referenced by these URIs and not retrieve them at the time they are used in a request.
// OPs can require that request_uri values used be pre-registered with the require_request_uri_registration
// discovery parameter.
func (c *OIDCClient) GetRequestURIs() []string {
	return c.RequestURIs
}

// GetJSONWebKeys returns the JSON Web Key Set containing the public keys used by the client to authenticate.
func (c *OIDCClient) GetJSONWebKeys() *jose.JSONWebKeySet {
	return c.JSONWebKeys
}

// GetJSONWebKeys returns the URL for lookup of JSON Web Key Set containing the
// public keys used by the client to authenticate.
func (c *OIDCClient) GetJSONWebKeysURI() string {
	return c.JSONWebKeysURI
}

// JWS [JWS] alg algorithm [JWA] that MUST be used for signing OAuthRequestWrapper Objects sent to the OP.
// All OAuthRequestWrapper Objects from this OAuthClient MUST be rejected, if not signed with this algorithm.
func (c *OIDCClient) GetRequestObjectSigningAlgorithm() string {
	return c.RequestObjectSigningAlgorithm
}

// Requested OAuthClient Authentication method for the Token Endpoint. The options are client_secret_post,
// client_secret_basic, client_secret_jwt, private_key_jwt, and none.
func (c *OIDCClient) GetTokenEndpointAuthMethod() string {
	return c.TokenEndpointAuthMethod
}

// JWS [JWS] alg algorithm [JWA] that MUST be used for signing the JWT [JWT] used to authenticate the
// OAuthClient at the Token Endpoint for the private_key_jwt and client_secret_jwt authentication methods.
func (c *OIDCClient) GetTokenEndpointAuthSigningAlgorithm() string {
	return "RS256"
}
