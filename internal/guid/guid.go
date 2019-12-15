package guid

import (
	"errors"
	"net/url"
	"strings"
)

type GUID struct {
	db  string
	id  string
	rev string
	s   string
}

func New(db string, id string) GUID {
	return GUID{
		db: db,
		id: id,
	}
}

func NewWithRev(db string, id string, rev string) GUID {
	return GUID{
		db:  db,
		id:  id,
		rev: rev,
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
		return GUID{}, errors.New("is missing ID")
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

	if rev := query.Get("rev"); rev != "" {
		return NewWithRev(db, id, rev), nil
	}
	return New(db, id), nil
}

func (g GUID) DB() string {
	return g.db
}

func (g GUID) ID() string {
	return g.id
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
		if g.rev != "" {
			s.WriteString("?rev=")
			s.WriteString(g.rev)
		}
		g.s = s.String()
	}

	return g.s
}
