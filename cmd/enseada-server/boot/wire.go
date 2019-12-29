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
