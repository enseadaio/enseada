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
	"github.com/stretchr/testify/require"
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
			require.NoError(t, err)
			assert.Equal(t, tt.out, v.Components)
		})
	}
}

var invalidParseTests = []struct {
	in  string
	err error
}{
	{in: "", err: fmt.Errorf("illegal version string: ")},
	{in: "not-a-version", err: fmt.Errorf("illegal version string: not-a-version")},
	{in: "not.a.version", err: fmt.Errorf("illegal version string: not.a.version")},
	{in: "1.0.Final", err: fmt.Errorf("invalid version 1.0.Final, qualifiers must be preceded by a '-' character")},
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

func BenchmarkParse(b *testing.B) {
	for _, tt := range validParseTests {
		b.Run(tt.in, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				v, err := Parse(tt.in)
				require.NoError(b, err)
				assert.Equal(b, tt.out, v.Components)
			}
		})
	}
}
