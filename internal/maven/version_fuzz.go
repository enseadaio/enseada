// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// +build gofuzz

package maven

func Fuzz(data []byte) int {
	v, err := Parse(string(data))
	if err != nil {
		if v != nil {
			panic("v != nil on error")
		}
		return 0
	}

	if len(v.Components) == 0 {
		return 0
	}

	return 1
}
