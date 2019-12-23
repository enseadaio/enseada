package oidc

import (
	"github.com/enseadaio/enseada/internal/auth/oauth"
	"github.com/enseadaio/enseada/internal/couch"
)

type Session struct {
	ID       string         `json:"_id,omitempty"`
	Rev      string         `json:"_rev,omitempty"`
	Kind     couch.Kind     `json:"kind"`
	AuthCode string         `json:"auth_code"`
	Req      *oauth.Request `json:"req"`
}
