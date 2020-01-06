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
	{"1", ListComponent{IntComponent(1)}},
	{"1.5", ListComponent{IntComponent(1), IntComponent(5)}},
	{"1.5-", ListComponent{IntComponent(1), IntComponent(5)}},
	{"1.5.2", ListComponent{IntComponent(1), IntComponent(5), IntComponent(2)}},
	{"1.0-snapshot", ListComponent{IntComponent(1), ListComponent{StringComponent("snapshot")}}},
	{"1.0-snapshot.", ListComponent{IntComponent(1), ListComponent{StringComponent("snapshot")}}},
	{"1.0-alpha-1", ListComponent{IntComponent(1), ListComponent{StringComponent("alpha")}, ListComponent{IntComponent(1)}}},
	{"1.0-alpha-1-", ListComponent{IntComponent(1), ListComponent{StringComponent("alpha")}, ListComponent{IntComponent(1)}}},
	{"1.0-alpha10-snapshot", ListComponent{IntComponent(1), ListComponent{StringComponent("alpha"), IntComponent(10)}, ListComponent{StringComponent("snapshot")}}},
	{"1.0-1", ListComponent{IntComponent(1), ListComponent{IntComponent(1)}}},
	{"1.0alpha1", ListComponent{IntComponent(1), IntComponent(0), StringComponent("alpha"), IntComponent(1)}},
	{"1alpha1", ListComponent{IntComponent(1), StringComponent("alpha"), IntComponent(1)}},
	{"1.0.Final", ListComponent{IntComponent(1), IntComponent(0), StringComponent("final")}},
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

func TestParseInvalid(t *testing.T) {
	for _, tt := range []struct {
		in  string
		err error
	}{
		{"", fmt.Errorf("illegal version string: ")},
		{"not-a-version", fmt.Errorf("illegal version string: not-a-version")},
		{"not.a.version", fmt.Errorf("illegal version string: not.a.version")},
	} {
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

func TestCompare(t *testing.T) {
	for _, tt := range []struct {
		l   string
		r   string
		res int
	}{
		{"1", "1.0", 0},
		{"1", "1.0-ga", 0},
		{"1", "1.0-final", 0},
		{"1-a", "1.0-alpha", 0},
		{"1a", "1.alpha", 0},
		{"1b", "1.beta", 0},
		{"1b-2", "1.beta-1", 1},
		{"1m-1", "1.milestone-1", 0},
		{"1m-1", "1.milestone-1.0", 0},
		{"1m.1", "1.milestone.1", 0},
		{"1m.1", "1.milestone.1.0", 0},
		{"1-m.1", "1.0-milestone.1", 0},
		{"1-m.1", "1.0-milestone.1.0", 0},
		{"1", "1.0-alpha", 1},
		{"1", "1.0-beta", 1},
		{"1", "1.0-milestone", 1},
		{"1", "1.0-rc", 1},
		{"1", "1.0-snapshot", 1},
		{"1", "1.0-whatever", -1},
	} {
		t.Run(fmt.Sprintf("%s vs %v", tt.l, tt.r), func(t *testing.T) {
			v1, err := Parse(tt.l)
			require.NoError(t, err)
			v2, err := Parse(tt.r)
			require.NoError(t, err)

			res := v1.Compare(v2)
			assert.Equal(t, tt.res, res)
		})
	}
}

func TestIntComponent_Compare(t *testing.T) {
	for _, tt := range []struct {
		l   IntComponent
		r   VersionComponent
		res int
	}{
		{IntComponent(1), IntComponent(1), 0},
		{IntComponent(2), IntComponent(1), 1},
		{IntComponent(3), IntComponent(4), -1},
		{IntComponent(4), StringComponent("test"), 1},
		{IntComponent(5), ListComponent{}, 1},
		{IntComponent(0), nil, 0},
		{IntComponent(6), nil, 1},
	} {
		t.Run(fmt.Sprintf("%d vs %v", tt.l, tt.r), func(t *testing.T) {
			res := tt.l.Compare(tt.r)
			assert.Equal(t, tt.res, res)
		})
	}
}

func TestStringComponent_Compare(t *testing.T) {
	for _, tt := range []struct {
		l   StringComponent
		r   VersionComponent
		res int
	}{
		{StringComponent("test"), IntComponent(1), -1},
		{StringComponent("test"), ListComponent{}, -1},
		{StringComponent("test"), StringComponent(""), 1},
		{StringComponent("test"), nil, 1},
		{StringComponent(""), StringComponent(""), 0},
		{StringComponent(""), StringComponent("alpha"), 1},
		{StringComponent(""), StringComponent("beta"), 1},
		{StringComponent(""), StringComponent("milestone"), 1},
		{StringComponent(""), StringComponent("rc"), 1},
		{StringComponent(""), StringComponent("snapshot"), 1},
		{StringComponent(""), StringComponent("xyz"), -1},
	} {
		t.Run(fmt.Sprintf("%s vs %v", tt.l, tt.r), func(t *testing.T) {
			res := tt.l.Compare(tt.r)
			assert.Equal(t, tt.res, res)
		})
	}
}

func TestListComponent_Compare(t *testing.T) {
	for _, tt := range []struct {
		l   ListComponent
		r   VersionComponent
		res int
	}{
		{ListComponent{}, IntComponent(1), -1},
		{ListComponent{}, StringComponent(""), 1},
		{ListComponent{IntComponent(1)}, ListComponent{IntComponent(2)}, -1},
		{ListComponent{IntComponent(1), IntComponent(0)}, ListComponent{IntComponent(1), IntComponent(2)}, -1},
		{ListComponent{IntComponent(1), IntComponent(0)}, ListComponent{IntComponent(2)}, -1},
		{ListComponent{IntComponent(1)}, ListComponent{IntComponent(1), IntComponent(2)}, -1},
		{ListComponent{IntComponent(2)}, ListComponent{IntComponent(1), IntComponent(2)}, 1},
		{ListComponent{IntComponent(1)}, ListComponent{IntComponent(1)}, 0},
		{ListComponent{IntComponent(2)}, ListComponent{IntComponent(1)}, 1},
		{ListComponent{}, nil, 0},
		{ListComponent{}, ListComponent{}, 0},
		{ListComponent{IntComponent(1)}, ListComponent{IntComponent(1), StringComponent("alpha")}, 1},
		{ListComponent{IntComponent(1)}, ListComponent{IntComponent(1), StringComponent("xyz")}, -1},
	} {
		t.Run(fmt.Sprintf("%v vs %v", tt.l, tt.r), func(t *testing.T) {
			res := tt.l.Compare(tt.r)
			assert.Equal(t, tt.res, res)
		})
	}
}
