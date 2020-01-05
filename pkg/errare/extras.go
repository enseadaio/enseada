// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package errare

type Extras map[string]interface{}

func (e Extras) GetOr(k string, def interface{}) interface{} {
	v, ok := e[k]
	if !ok {
		return def
	}
	return v
}
