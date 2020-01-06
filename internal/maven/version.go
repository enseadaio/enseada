// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"fmt"
	"strconv"
	"strings"
)

const (
	dotRune  rune = '.'
	dashRune rune = '-'
)

type VersionComponent interface {
	Compare(oc VersionComponent) int
}

type IntComponent uint64

func (i IntComponent) Compare(oc VersionComponent) int {
	return 0
}

type StringComponent string

func (s StringComponent) Compare(oc VersionComponent) int {
	return 0
}

type ListComponent []VersionComponent

func (l ListComponent) Compare(oc VersionComponent) int {
	return 0
}

type Version struct {
	Components ListComponent
}

func (v *Version) Compare(ov *Version) int {
	return 0
}

func Parse(v string) (*Version, error) {
	vv := []rune(v)
	if len(vv) == 0 || isSeparator(vv[0]) {
		return nil, fmt.Errorf("illegal version string: %s", v)
	}

	ver := &Version{}

	cc := make(ListComponent, 0)

	b := &strings.Builder{}
	var pr rune
	for _, r := range vv {
		d := string(r)
		if isSeparator(r) && d != "" {
			cc = appendBuffer(cc, b)
			if r == dashRune {
				if ver.Components == nil {
					ver.Components = cc
				} else {
					ver.Components = append(ver.Components, cc)
				}
				cc = make([]VersionComponent, 0)
			}
			continue
		}

		if pr == 0 {
			b.WriteRune(r)
			pr = r
			continue
		}

		_, perr := strconv.Atoi(string(pr))
		_, err := strconv.Atoi(string(r))
		pr = r

		if perr == nil {
			if err == nil {
				b.WriteRune(r)
				continue
			}
			cc = appendBuffer(cc, b)
			b.WriteRune(r)
			continue
		}

		if err != nil {
			b.WriteRune(r)
			continue
		}

		cc = appendBuffer(cc, b)
		b.WriteRune(r)
	}
	cc = appendBuffer(cc, b)
	if ver.Components == nil {
		ver.Components = cc
	} else {
		ver.Components = append(ver.Components, cc)
	}

	return ver, nil
}

func isSeparator(r rune) bool {
	return r == dotRune || r == dashRune
}

func appendBuffer(cc []VersionComponent, b *strings.Builder) []VersionComponent {
	s := b.String()
	b.Reset()
	if n, err := strconv.Atoi(s); err == nil {
		return append(cc, IntComponent(n))
	} else if len(s) > 0 {
		return append(cc, StringComponent(s))
	} else {
		return cc
	}
}
