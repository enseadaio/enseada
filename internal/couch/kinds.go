// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
	KindUser               = Kind("user")
)
