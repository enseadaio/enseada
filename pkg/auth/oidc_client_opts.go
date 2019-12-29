package auth

import (
	"gopkg.in/square/go-jose.v2"
)

type OIDCClientOption func(opts *OIDCClientOptions)

type OIDCClientOptions struct {
	OAuthClientOptions []OAuthClientOption
	JwksURI            string
	Jwks               *jose.JSONWebKeySet
	TokenAuthMethod    string
	RequestURIs        []string
	RequestAlgo        string
}

func OIDCRedirectURIs(uris ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthRedirectURIs(uris...))
	}
}

func OIDCGrantTypes(types ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthGrantTypes(types...))
	}
}

func OIDCResponseTypes(types ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthResponseTypes(types...))
	}
}

func OIDCScopes(scopes ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthScopes(scopes...))
	}
}

func OIDCAudiences(audiences ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthAudiences(audiences...))
	}
}

func OIDCPublic(public bool) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.OAuthClientOptions = append(opts.OAuthClientOptions, OAuthPublic(public))
	}
}

func OIDCJSONWebKeysURI(uri string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.JwksURI = uri
	}
}

func OIDCJSONWebKeys(jwks *jose.JSONWebKeySet) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.Jwks = jwks
	}
}

func OIDCTokenEndpointAuthMethod(method string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.TokenAuthMethod = method
	}
}

func OIDCRequestURIs(uris ...string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.RequestURIs = uris
	}
}

func OIDCRequestObjectSigningAlgorithm(algorithm string) OIDCClientOption {
	return func(opts *OIDCClientOptions) {
		opts.RequestAlgo = algorithm
	}
}
