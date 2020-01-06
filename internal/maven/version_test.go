// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

var validParseTests = []struct {
	in  string
	out ListComponent
}{
	{in: "1", out: ListComponent{IntComponent(1)}},
	{in: "1.5", out: ListComponent{IntComponent(1), IntComponent(5)}},
	{in: "1.5.2", out: ListComponent{IntComponent(1), IntComponent(5), IntComponent(2)}},
	{in: "1.0-snapshot", out: ListComponent{IntComponent(1), IntComponent(0), ListComponent{StringComponent("snapshot")}}},
	{in: "1.0-alpha-1", out: ListComponent{IntComponent(1), IntComponent(0), ListComponent{StringComponent("alpha")}, ListComponent{IntComponent(1)}}},
	{in: "1.0-alpha10-snapshot", out: ListComponent{IntComponent(1), IntComponent(0), ListComponent{StringComponent("alpha"), IntComponent(10)}, ListComponent{StringComponent("snapshot")}}},
	{in: "1.0-1", out: ListComponent{IntComponent(1), IntComponent(0), ListComponent{IntComponent(1)}}},
}

func TestParseValid(t *testing.T) {
	for _, tt := range validParseTests {
		t.Run(tt.in, func(t *testing.T) {
			v, err := Parse(tt.in)
			assert.NoError(t, err)
			assert.Equal(t, tt.out, v.Components)
		})
	}
}

var invalidParseTests = []struct {
	in  string
	err error
}{
	{in: "", err: fmt.Errorf("illegal version string: ")},
}

func TestParseInvalid(t *testing.T) {
	for _, tt := range invalidParseTests {
		t.Run(tt.in, func(t *testing.T) {
			v, err := Parse(tt.in)
			assert.Error(t, err)
			assert.Equal(t, tt.err, err)
			assert.Nil(t, v)
		})
	}
}
