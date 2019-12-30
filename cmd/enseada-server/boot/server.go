// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
