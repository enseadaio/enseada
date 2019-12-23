package oauth

import (
	"errors"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/ory/fosite"
	"golang.org/x/crypto/bcrypt"
)

type Client struct {
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

func NewClient(id string, secret string, opts ...ClientOption) (*Client, error) {
	if id == "" {
		return nil, errors.New("client ID cannot be empty")
	}

	options := ClientOptions{
		redirectURIs:  []string{},
		grantTypes:    []string{},
		responseTypes: []string{},
		scopes:        []string{},
		audiences:     []string{},
		public:        false,
	}
	for _, opt := range opts {
		opt(&options)
	}

	if !options.public && secret == "" {
		return nil, errors.New("client secret cannot be empty for non-public clients")
	}

	var hashed []byte
	if !options.public {
		h, err := bcrypt.GenerateFromPassword([]byte(secret), bcrypt.DefaultCost)
		if err != nil {
			return nil, err
		}
		hashed = h
	}

	if len(options.grantTypes) == 0 {
		options.grantTypes = fosite.Arguments{"authorization_code"}
	}

	if len(options.responseTypes) == 0 {
		options.responseTypes = fosite.Arguments{"code"}
	}

	return &Client{
		ID:            id,
		Kind:          couch.KindOAuthClient,
		HashedSecret:  hashed,
		RedirectURIs:  options.redirectURIs,
		GrantTypes:    options.grantTypes,
		ResponseTypes: options.responseTypes,
		Scopes:        options.scopes,
		Audiences:     options.audiences,
		Public:        options.public,
	}, nil
}

// GetID returns the client ID.
func (c *Client) GetID() string {
	return c.ID
}

func (c *Client) GetRev() string {
	return c.Rev
}

func (c *Client) SetRev(rev string) {
	c.Rev = rev
}

// GetHashedSecret returns the hashed secret as it is stored in the store.
func (c *Client) GetHashedSecret() []byte {
	return c.HashedSecret
}

// GetRedirectURIs returns the client's allowed redirect URIs.
func (c *Client) GetRedirectURIs() []string {
	return c.RedirectURIs
}

// GetGrantTypes returns the client's allowed grant types.
func (c *Client) GetGrantTypes() fosite.Arguments {
	return c.GrantTypes
}

// GetResponseTypes returns the client's allowed response types.
func (c *Client) GetResponseTypes() fosite.Arguments {
	return c.ResponseTypes
}

// GetScopes returns the scopes this client is allowed to request.
func (c *Client) GetScopes() fosite.Arguments {
	return c.Scopes
}

// IsPublic returns true, if this client is marked as public.
func (c *Client) IsPublic() bool {
	return c.Public
}

// GetAudience returns the allowed audience(s) for this client.
func (c *Client) GetAudience() fosite.Arguments {
	return c.Audiences
}
