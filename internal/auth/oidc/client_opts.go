package oidc

import (
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"gopkg.in/square/go-jose.v2"
)

type ClientOption func(opts *ClientOptions)
type ClientOptions struct {
	oauthOpts       []oauth.ClientOption
	jwksURI         string
	jwks            *jose.JSONWebKeySet
	tokenAuthMethod string
	requestURIs     []string
	requestAlgo     string
}

func RedirectURIs(uris ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.RedirectURIs(uris...))
	}
}

func GrantTypes(types ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.GrantTypes(types...))
	}
}

func ResponseTypes(types ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.ResponseTypes(types...))
	}
}

func Scopes(scopes ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.Scopes(scopes...))
	}
}

func Audiences(audiences ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.Audiences(audiences...))
	}
}

func Public(public bool) ClientOption {
	return func(opts *ClientOptions) {
		opts.oauthOpts = append(opts.oauthOpts, oauth.Public(public))
	}
}

func JSONWebKeysURI(uri string) ClientOption {
	return func(opts *ClientOptions) {
		opts.jwksURI = uri
	}
}

func JSONWebKeys(jwks *jose.JSONWebKeySet) ClientOption {
	return func(opts *ClientOptions) {
		opts.jwks = jwks
	}
}

func TokenEndpointAuthMethod(method string) ClientOption {
	return func(opts *ClientOptions) {
		opts.tokenAuthMethod = method
	}
}

func RequestURIs(uris ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.requestURIs = uris
	}
}

func RequestObjectSigningAlgorithm(algorithm string) ClientOption {
	return func(opts *ClientOptions) {
		opts.requestAlgo = algorithm
	}
}
