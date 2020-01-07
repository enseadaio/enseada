// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"database/sql"
	"fmt"
	"strconv"
	"strings"
	"text/scanner"
	"unicode"
)

const (
	dotRune            rune = '.'
	dashRune           rune = '-'
	QualifierAlpha          = "alpha"
	QualifierBeta           = "beta"
	QualifierMilestone      = "milestone"
	QualifierRC             = "rc"
	QualifierSnapshot       = "snapshot"
	QualifierRelease        = ""
	QualifierSpecial        = "sp"
)

var wellKnownQualifiers = [7]string{
	QualifierAlpha,
	QualifierBeta,
	QualifierMilestone,
	QualifierRC,
	QualifierSnapshot,
	QualifierRelease,
	QualifierSpecial,
}

var wellKnownAliases = map[string]string{
	QualifierAlpha:     QualifierAlpha,
	"a":                QualifierAlpha,
	QualifierBeta:      QualifierBeta,
	"b":                QualifierBeta,
	QualifierMilestone: QualifierMilestone,
	"m":                QualifierMilestone,
	QualifierRC:        QualifierRC,
	"cr":               QualifierRC,
	QualifierSnapshot:  QualifierSnapshot,
	QualifierRelease:   QualifierRelease,
	"ga":               QualifierRelease,
	"final":            QualifierRelease,
	"release":          QualifierRelease,
}

type VersionComponent interface {
	Compare(oc VersionComponent) int
	isEmpty() bool
}

type IntComponent uint64

func (i IntComponent) Compare(oc VersionComponent) int {
	if oc == nil {
		if i == 0 {
			return 0
		}
		return 1
	}

	switch oc.(type) {
	case IntComponent:
		j := oc.(IntComponent)
		if i == j {
			return 0
		}
		if i < j {
			return -1
		}
		return 1
	case StringComponent:
		return 1
	case ListComponent:
		return 1
	default:
		panic(fmt.Sprintf("%v is not one of the sealed implementations", oc))
	}
}

func (i IntComponent) isEmpty() bool {
	return i == 0
}

type StringComponent string

func (s StringComponent) Compare(oc VersionComponent) int {
	if oc == nil {
		oc = StringComponent("")
	}

	switch oc.(type) {
	case IntComponent:
		return -1
	case StringComponent:
		left := string(s)
		right := string(oc.(StringComponent))
		w1, ok := wellKnownAliases[left]
		if ok {
			left = comparableStringQualifier(w1)
		}

		w2, ok := wellKnownAliases[right]
		if ok {
			right = comparableStringQualifier(w2)
		}
		if left == right {
			return 0
		}
		if left < right {
			return -1
		}
		return 1
	case ListComponent:
		return -1
	default:
		panic(fmt.Sprintf("%v is not one of the sealed implementations", oc))
	}
}

func (s StringComponent) isEmpty() bool {
	return s == ""
}

func comparableStringQualifier(s string) string {
	for i, q := range wellKnownQualifiers {
		if q == s {
			return string(i)
		}
	}

	return fmt.Sprintf("%d-%s", len(wellKnownQualifiers), s)
}

type ListComponent []VersionComponent

func (l ListComponent) Compare(oc VersionComponent) int {
	if oc == nil {
		if len(l) == 0 {
			return 0
		}
		oc = ListComponent{}
	}

	switch oc.(type) {
	case IntComponent:
		return -1
	case StringComponent:
		return 1
	case ListComponent:
		left := l
		right := oc.(ListComponent)
		if len(left) < len(right) {
			return right.Compare(left) * -1
		}

		for i, c1 := range left {
			var c2 VersionComponent
			if len(right)-1 >= i {
				c2 = right[i]
			}
			if r := c1.Compare(c2); r != 0 {
				return r
			}
		}
		return 0
	default:
		panic(fmt.Sprintf("%v is not one of the sealed implementations", oc))
	}
}

func (l ListComponent) Normalize() ListComponent {
	nl := l
	for i := len(nl) - 1; i >= 0; i-- {
		c := nl[i]
		_, isList := c.(ListComponent)
		if c == nil || c.isEmpty() {
			nl = append(nl[:i], nl[i+1:]...)
		} else if !isList {
			break
		}
	}
	return nl
}

func (l ListComponent) isEmpty() bool {
	return len(l) == 0
}

type Version struct {
	Components ListComponent
	s          string
	isSnapshot sql.NullBool
}

func (v *Version) Compare(ov *Version) int {
	return v.Components.Compare(ov.Components)
}

func (v *Version) String() string {
	return v.s
}

func (v *Version) IsSnapshot() bool {
	if !v.isSnapshot.Valid {
		v.isSnapshot.Bool, v.isSnapshot.Valid = strings.Contains(strings.ToLower(v.String()), "snapshot"), true
	}
	return v.isSnapshot.Bool
}

func ParseVersion(v string) (*Version, error) {
	if v == "" || !unicode.IsDigit(rune(v[0])) {
		return nil, fmt.Errorf("illegal version string: %s", v)
	}

	ver := &Version{
		s: v,
	}

	cc := make(ListComponent, 0)
	b := &strings.Builder{}
	var pr rune
	rr := []rune(v)
	for i, r := range rr {
		nxt := peek(rr, i)
		if isSeparator(r) && nxt == scanner.EOF {
			break
		}

		isDigToLet := i > 0 && unicode.IsDigit(rr[i-1]) && unicode.IsLetter(r)
		if isSeparator(r) || isDigToLet {
			cc = appendBuffer(cc, b)
			if r == dashRune {
				cc = cc.Normalize()
				if ver.Components == nil {
					ver.Components = cc
				} else {
					ver.Components = append(ver.Components, cc)
				}
				cc = make(ListComponent, 0)
			}

			if isDigToLet {
				b.WriteRune(r)
				pr = r
			}

			continue
		}

		if pr == 0 {
			b.WriteRune(r)
			pr = r
			continue
		}

		pn := unicode.IsDigit(pr)
		n := unicode.IsDigit(r)
		pr = r

		if pn {
			if n {
				b.WriteRune(r)
				continue
			}
			cc = appendBuffer(cc, b)
			b.WriteRune(r)
			continue
		}

		if !n {
			b.WriteRune(r)
			continue
		}

		cc = appendBuffer(cc, b)
		b.WriteRune(r)
	}
	cc = appendBuffer(cc, b).Normalize()
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

func peek(rr []rune, i int) rune {
	if len(rr)-1 > i {
		return rr[i+1]
	}
	return scanner.EOF
}

func appendBuffer(cc ListComponent, b *strings.Builder) ListComponent {
	s := strings.ToLower(b.String())
	b.Reset()
	if n, err := strconv.Atoi(s); err == nil {
		return append(cc, IntComponent(n))
	} else if len(s) > 0 {
		return append(cc, StringComponent(s))
	} else {
		return cc
	}
}
