// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

type OAuthClientOption func(opts *OAuthClientOptions)

func OAuthRedirectURIs(uris ...string) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.RedirectURIs = uris
	}
}

func OAuthGrantTypes(types ...string) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.GrantTypes = types
	}
}

func OAuthResponseTypes(types ...string) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.ResponseTypes = types
	}
}

func OAuthScopes(scopes ...string) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.Scopes = scopes
	}
}

func OAuthAudiences(audiences ...string) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.Audiences = audiences
	}
}

func OAuthPublic(public bool) OAuthClientOption {
	return func(opts *OAuthClientOptions) {
		opts.Public = public
	}
}
