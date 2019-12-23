package oidc

import (
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"gopkg.in/square/go-jose.v2"
)

type OpenIDConnectClient struct {
	*oauth.Client
	JSONWebKeysURI                string              `json:"jwks_uri"`
	JSONWebKeys                   *jose.JSONWebKeySet `json:"jwks"`
	TokenEndpointAuthMethod       string              `json:"token_endpoint_auth_method"`
	RequestURIs                   []string            `json:"request_uris"`
	RequestObjectSigningAlgorithm string              `json:"request_object_signing_alg"`
}

func NewOpenIDConnectClient(id string, secret string, opts ...ClientOption) (*OpenIDConnectClient, error) {
	options := ClientOptions{
		oauthOpts:       []oauth.ClientOption{},
		jwksURI:         "",
		jwks:            &jose.JSONWebKeySet{},
		tokenAuthMethod: "",
		requestURIs:     []string{},
		requestAlgo:     "",
	}
	for _, opt := range opts {
		opt(&options)
	}

	oc, err := oauth.NewClient(id, secret, options.oauthOpts...)
	if err != nil {
		return nil, err
	}

	return &OpenIDConnectClient{
		Client:                        oc,
		JSONWebKeysURI:                options.jwksURI,
		JSONWebKeys:                   options.jwks,
		TokenEndpointAuthMethod:       options.tokenAuthMethod,
		RequestURIs:                   options.requestURIs,
		RequestObjectSigningAlgorithm: options.requestAlgo,
	}, nil
}

// GetRequestURIs is an array of request_uri values that are pre-registered by the RP for use at the OP. Servers MAY
// cache the contents of the files referenced by these URIs and not retrieve them at the time they are used in a request.
// OPs can require that request_uri values used be pre-registered with the require_request_uri_registration
// discovery parameter.
func (c *OpenIDConnectClient) GetRequestURIs() []string {
	return c.RequestURIs
}

// GetJSONWebKeys returns the JSON Web Key Set containing the public keys used by the client to authenticate.
func (c *OpenIDConnectClient) GetJSONWebKeys() *jose.JSONWebKeySet {
	return c.JSONWebKeys
}

// GetJSONWebKeys returns the URL for lookup of JSON Web Key Set containing the
// public keys used by the client to authenticate.
func (c *OpenIDConnectClient) GetJSONWebKeysURI() string {
	return c.JSONWebKeysURI
}

// JWS [JWS] alg algorithm [JWA] that MUST be used for signing RequestWrapper Objects sent to the OP.
// All RequestWrapper Objects from this Client MUST be rejected, if not signed with this algorithm.
func (c *OpenIDConnectClient) GetRequestObjectSigningAlgorithm() string {
	return c.RequestObjectSigningAlgorithm
}

// Requested Client Authentication method for the Token Endpoint. The options are client_secret_post,
// client_secret_basic, client_secret_jwt, private_key_jwt, and none.
func (c *OpenIDConnectClient) GetTokenEndpointAuthMethod() string {
	return c.TokenEndpointAuthMethod
}

// JWS [JWS] alg algorithm [JWA] that MUST be used for signing the JWT [JWT] used to authenticate the
// Client at the Token Endpoint for the private_key_jwt and client_secret_jwt authentication methods.
func (c *OpenIDConnectClient) GetTokenEndpointAuthSigningAlgorithm() string {
	return "RS256"
}
