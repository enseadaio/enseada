package auth

import (
	"github.com/enseadaio/enseada/internal/couch"
)

type OIDCSession struct {
	ID       string        `json:"_id,omitempty"`
	Rev      string        `json:"_rev,omitempty"`
	Kind     couch.Kind    `json:"kind"`
	AuthCode string        `json:"auth_code"`
	Req      *OAuthRequest `json:"req"`
}
