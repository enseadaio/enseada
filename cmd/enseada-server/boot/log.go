// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package boot

import (
	"strings"

	"github.com/labstack/gommon/log"
	"github.com/spf13/viper"
)

func logLvl(conf *viper.Viper) log.Lvl {
	return getLogLvl(conf.GetString("log.level"))
}

func getLogLvl(lvl string) log.Lvl {
	switch strings.ToUpper(lvl) {
	case "DEBUG":
		return log.DEBUG
	case "INFO":
		return log.INFO
	case "WARN":
		return log.WARN
	case "ERROR":
		return log.ERROR
	case "OFF":
		return log.OFF
	default:
		return log.INFO
	}
}
