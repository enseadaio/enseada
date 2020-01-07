// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package storage

import (
	"context"
	"fmt"

	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/pkg/log"
)

type Config struct {
	Provider string
	Local    LocalConfig
	S3       S3Config
	Azure    AzureConfig
	GCS      GCSConfig
}

type LocalConfig struct {
	StorageDir string
}

type S3Config struct {
	Bucket   string
	Prefix   string
	Region   string
	Endpoint string
	SSE      string
}

type AzureConfig struct {
	Bucket string
	Prefix string
}

type GCSConfig struct {
	Bucket string
	Prefix string
}

var (
	ErrMissingConfig = func(provider string, param string) error {
		return fmt.Errorf("missing %s configuration for provider %s", param, provider)
	}
)

type Module struct {
	logger  log.Logger
	Backend storage.Backend
}

func NewModule(_ context.Context, logger log.Logger, c Config) (*Module, error) {
	var b storage.Backend
	switch c.Provider {
	case "azure":
		ac := c.Azure
		if ac.Bucket == "" {
			return nil, ErrMissingConfig(c.Provider, "bucket")
		}
		b = storage.NewMicrosoftBlobBackend(ac.Bucket, ac.Prefix)
	case "gcs":
		gc := c.GCS
		if gc.Bucket == "" {
			return nil, ErrMissingConfig(c.Provider, "bucket")
		}
		b = storage.NewGoogleCSBackend(gc.Bucket, gc.Prefix)
	case "s3":
		sc := c.S3
		if sc.Bucket == "" {
			return nil, ErrMissingConfig(c.Provider, "bucket")
		}
		if sc.Region == "" {
			return nil, ErrMissingConfig(c.Provider, "region")
		}
		b = storage.NewAmazonS3Backend(sc.Bucket, sc.Prefix, sc.Region, sc.Endpoint, sc.SSE)
	case "local":
		b = storage.NewLocalFilesystemBackend(c.Local.StorageDir)
	default:
		return nil, fmt.Errorf("unsupported storage provider: %s", c.Provider)
	}
	return &Module{
		logger:  logger,
		Backend: b,
	}, nil
}

func (m *Module) Start(ctx context.Context) error {
	m.logger.Info("started storage module")
	return nil
}

func (m *Module) Stop(ctx context.Context) error {
	m.logger.Info("stopped storage module")
	return nil
}
