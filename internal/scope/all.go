// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package scope

var All = []string{
	// Global
	OpenID,
	Profile,

	// ACL
	ACLPermissionRead,
	ACLPermissionWrite,
	ACLPermissionDelete,

	// OAuth
	OAuthClientRead,
	OAuthClientWrite,

	// Maven
	MavenRepoRead,
	MavenRepoWrite,
	MavenFileRead,
	MavenFileWrite,
}
