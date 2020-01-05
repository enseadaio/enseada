// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package enseada

import "fmt"

const (
	VersionMajor  = 0
	VersionMinor  = 1
	VersionPatch  = 0
	VersionSuffix = "-DEVEL"
)

func VersionString() string {
	return fmt.Sprintf("%d.%d.%d%s", VersionMajor, VersionMinor, VersionPatch, VersionSuffix)
}
