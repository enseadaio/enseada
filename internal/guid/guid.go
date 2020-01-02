// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package guid

import (
	"errors"
	"net/url"
	"strings"

	"github.com/enseadaio/enseada/internal/couch"
)

type GUID struct {
	db   string
	id   string
	rev  string
	kind couch.Kind
	s    string
}

func New(db string, id string, kind couch.Kind) GUID {
	return GUID{
		db:   db,
		id:   id,
		kind: kind,
	}
}

func NewWithRev(db string, id string, kind couch.Kind, rev string) GUID {
	return GUID{
		db:   db,
		id:   id,
		kind: kind,
		rev:  rev,
	}
}

func Parse(guid string) (GUID, error) {
	if guid == "" {
		return GUID{}, errors.New("GUID can't be blank")
	}

	if !strings.Contains(guid, "://") {
		return GUID{}, errors.New("is missing database")
	}

	s1 := strings.Split(guid, "://")
	db := s1[0]
	id := s1[1]
	if id == "" {
		return GUID{}, errors.New("is missing Username")
	}

	query := url.Values{}
	if strings.Contains(id, "?") {
		s2 := strings.Split(id, "?")
		id = s2[0]
		q, err := url.ParseQuery(s2[1])
		if err != nil {
			return GUID{}, err
		}
		query = q
	}

	k := query.Get("kind")
	if k == "" {
		return GUID{}, errors.New("is missing kind")
	}

	kind := couch.Kind(k)

	if rev := query.Get("rev"); rev != "" {
		return NewWithRev(db, id, kind, rev), nil
	}
	return New(db, id, kind), nil
}

func (g GUID) DB() string {
	return g.db
}

func (g GUID) ID() string {
	return g.id
}

func (g GUID) Kind() couch.Kind {
	return g.kind
}

func (g GUID) Rev() string {
	return g.rev
}

func (g GUID) String() string {
	if g.s == "" {
		var s strings.Builder
		s.WriteString(g.db)
		s.WriteString("://")
		s.WriteString(g.id)
		s.WriteString("?kind=")
		s.WriteString(string(g.kind))
		if g.rev != "" {
			s.WriteString("&rev=")
			s.WriteString(g.rev)
		}
		g.s = s.String()
	}

	return g.s
}
