// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
