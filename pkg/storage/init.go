package storage

import (
	"fmt"
	"github.com/chartmuseum/storage"
	"github.com/pkg/errors"
)

type Options struct {
	storageDir string
}

type Option func(opts *Options) error

func LocalDir(dir string) Option {
	return func(opts *Options) error {
		opts.storageDir = dir
		return nil
	}
}

func NewBackend(provider string, opts ...Option) (storage.Backend, error) {
	options := &Options{
		storageDir: "uploads",
	}

	for i := range opts {
		err := opts[i](options)
		if err != nil {
			return nil, err
		}
	}

	switch provider {
	//case "s3":
	//	return storage.NewAmazonS3Backend()
	case "local":
		return storage.NewLocalFilesystemBackend(options.storageDir), nil
	default:
		return nil, errors.New(fmt.Sprintf("unsupported storage provider: %s", provider))
	}
}
