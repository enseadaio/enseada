package boot

import (
	"fmt"

	"github.com/chartmuseum/storage"
	"github.com/spf13/viper"
)

func storageBackend(conf *viper.Viper) (storage.Backend, error) {
	provider := conf.GetString("storage.provider")
	storageDir := conf.GetString("storage.dir")

	switch provider {
	//case "s3":
	//	return storage.NewAmazonS3Backend()
	case "local":
		return storage.NewLocalFilesystemBackend(storageDir), nil
	default:
		return nil, fmt.Errorf("unsupported storage provider: %s", provider)
	}
}
