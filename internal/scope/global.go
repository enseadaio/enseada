package scope

const (
	OpenID  = "openid"
	Profile = "profile"
)

var AllScopes = []string{
	// Global
	OpenID,
	Profile,

	// ACL
	ACLRead,
	ACLWrite,
	ACLDelete,
}
