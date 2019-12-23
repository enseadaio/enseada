package oauth

type ClientOption func(opts *ClientOptions)
type ClientOptions struct {
	redirectURIs  []string
	grantTypes    []string
	responseTypes []string
	scopes        []string
	audiences     []string
	public        bool
}

func RedirectURIs(uris ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.redirectURIs = uris
	}
}

func GrantTypes(types ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.grantTypes = types
	}
}

func ResponseTypes(types ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.responseTypes = types
	}
}

func Scopes(scopes ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.scopes = scopes
	}
}

func Audiences(audiences ...string) ClientOption {
	return func(opts *ClientOptions) {
		opts.audiences = audiences
	}
}

func Public(public bool) ClientOption {
	return func(opts *ClientOptions) {
		opts.public = public
	}
}
