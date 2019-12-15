package acl

import (
	"context"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/go-kivik/kivik"
)

type ResourceManager struct {
	client *kivik.Client
}

func (m *ResourceManager) LoadResource(ctx context.Context, guid guid.GUID, dest interface{}) error {
	db := m.client.DB(ctx, guid.DB())
	options := kivik.Options{}

	if guid.Rev() != "" {
		options["_rev"] = guid.Rev()
	}

	row := db.Get(ctx, guid.ID(), options)
	return row.ScanDoc(dest)
}
