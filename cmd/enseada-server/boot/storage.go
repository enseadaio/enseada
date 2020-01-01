// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
