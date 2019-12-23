package couch

type Kind string

const (
	KindRepository         = Kind("repository")
	KindOAuthClient        = Kind("client")
	KindOAuthAuthorizeCode = Kind("authorize_code")
	KindOAuthAccessToken   = Kind("access_token")
	KindOAuthRefreshToken  = Kind("refresh_token")
	KindOpenIDSession      = Kind("refresh_token")
	KindPKCERequest        = Kind("pkce_request")
)
