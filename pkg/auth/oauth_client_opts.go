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
