// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"time"

	"github.com/enseadaio/enseada/internal/scope"
)

type UserConsent struct {
	Scopes         []string  `json:"scopes"`
	ConsentGivenAt time.Time `json:"consent_given_at"`
}

type User struct {
	Username       string                 `json:"_id"`
	Rev            string                 `json:"_rev,omitempty"`
	Password       string                 `json:"-"`
	HashedPassword []byte                 `json:"hashed_password"`
	Consent        map[string]UserConsent `json:"consent"`
}

func RootUser(pwd string) *User {
	c := UserConsent{
		Scopes:         scope.All,
		ConsentGivenAt: time.Now(),
	}

	return &User{
		Username: "root",
		Password: pwd,
		Consent: map[string]UserConsent{
			"enseada":     c,
			"enseada-cli": c,
		},
	}
}
