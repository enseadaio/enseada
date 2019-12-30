// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"context"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/spf13/viper"
)

func Boot(ctx context.Context, conf *viper.Viper) (*enseada.Server, error) {
	s, err := initServer(ctx, conf)
	if err != nil {
		return nil, err
	}

	s.Init()

	return s, nil
}
