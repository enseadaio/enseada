package boot

import (
	"github.com/labstack/gommon/log"
	"github.com/spf13/viper"
	"strings"
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
