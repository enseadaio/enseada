package boot

import (
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/spf13/viper"
)

func skb(conf *viper.Viper) enseada.SecretKeyBase {
	return enseada.SecretKeyBase(conf.GetString("secret.key.base"))
}

func publicHost(conf *viper.Viper) enseada.PublicHost {
	return enseada.PublicHost(viper.GetString("public.host"))
}
