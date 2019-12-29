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
