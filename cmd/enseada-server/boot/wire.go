// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//+build wireinject
//The build tag makes sure the stub is not built in the final build.

package boot

import (
	"context"

	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/google/wire"
	"github.com/spf13/viper"
)

var authSet = wire.NewSet(
	authLog,
	newOAuthClientStore,
	newOAuthRequestStore,
	newOIDCSessionStore,
	newPKCERequestStore,
	newStore,
	oauthProvider,
	defaultClientSecret,
	oauthClient,
)

var casbinSet = wire.NewSet(
	casbinLog,
	casbinModel,
	casbinAdapter,
	casbinWatcher,
	casbinEnforcer,
)

func initServer(ctx context.Context, c *viper.Viper) (*enseada.Server, error) {
	wire.Build(
		skb,
		publicHost,
		logLvl,
		dbClient,
		authSet,
		casbinSet,
		enseada.NewServer,
	)
	return &enseada.Server{}, nil
}
