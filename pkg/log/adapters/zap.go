// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package adapters

import (
	"fmt"

	"github.com/enseadaio/enseada/pkg/log"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type ZapLoggerAdapter struct {
	*zap.Logger
	lvl log.Level
}

func NewZapLoggerAdapter(lvl log.Level) (*ZapLoggerAdapter, error) {
	cfg := zap.NewProductionConfig()
	zlvl := new(zapcore.Level)
	err := zlvl.UnmarshalText([]byte(lvl))
	if err != nil {
		return nil, err
	}

	cfg.Level.SetLevel(*zlvl)
	cfg.OutputPaths = []string{"stdout"}

	l, err := cfg.Build(zap.AddCallerSkip(2))
	if err != nil {
		return nil, err
	}
	return &ZapLoggerAdapter{Logger: l, lvl: lvl}, nil
}

func (z *ZapLoggerAdapter) Log(lvl log.Level, i ...interface{}) {
	defer z.Sync()

	var msg string
	f, ok := i[0].(string)
	if !ok || f == "" {
		msg = fmt.Sprint(i...)
	} else {
		msg = fmt.Sprintf(f, i[1:]...)
	}

	switch lvl {
	case log.TRACE:
		z.Logger.Debug(msg)
	case log.DEBUG:
		z.Logger.Debug(msg)
	case log.INFO:
		z.Logger.Info(msg)
	case log.WARN:
		z.Logger.Warn(msg)
	case log.ERROR:
		z.Logger.Error(msg)
	case log.FATAL:
		z.Logger.Fatal(msg)
	case log.PANIC:
		z.Logger.Panic(msg)
	default:
		z.Logger.Info(msg)
	}
}

func (z *ZapLoggerAdapter) Logf(lvl log.Level, msg string, params ...interface{}) {
	defer z.Sync()

	s := fmt.Sprintf(msg, params...)
	switch lvl {
	case log.TRACE:
		z.Debug(s)
	case log.DEBUG:
		z.Debug(s)
	case log.INFO:
		z.Info(s)
	case log.WARN:
		z.Warn(s)
	case log.ERROR:
		z.Error(s)
	case log.FATAL:
		z.Fatal(s)
	case log.PANIC:
		z.Panic(s)
	default:
		z.Info(msg)
	}
}

func (z *ZapLoggerAdapter) Trace(i ...interface{}) {
	z.Log(log.TRACE, i...)
}

func (z *ZapLoggerAdapter) Tracef(msg string, params ...interface{}) {
	z.Logf(log.TRACE, msg, params...)
}

func (z *ZapLoggerAdapter) Debug(i ...interface{}) {
	z.Log(log.DEBUG, i...)
}

func (z *ZapLoggerAdapter) Debugf(msg string, params ...interface{}) {
	z.Logf(log.DEBUG, msg, params...)
}

func (z *ZapLoggerAdapter) Info(i ...interface{}) {
	z.Log(log.INFO, i...)
}

func (z *ZapLoggerAdapter) Infof(msg string, params ...interface{}) {
	z.Logf(log.INFO, msg, params...)
}

func (z *ZapLoggerAdapter) Warn(i ...interface{}) {
	z.Log(log.WARN, i...)
}

func (z *ZapLoggerAdapter) Warnf(msg string, params ...interface{}) {
	z.Logf(log.WARN, msg, params...)
}

func (z *ZapLoggerAdapter) Error(i ...interface{}) {
	z.Log(log.ERROR, i...)
}

func (z *ZapLoggerAdapter) Errorf(msg string, params ...interface{}) {
	z.Logf(log.ERROR, msg, params...)
}

func (z *ZapLoggerAdapter) Fatal(i ...interface{}) {
	z.Log(log.FATAL, i...)
}

func (z *ZapLoggerAdapter) Fatalf(msg string, params ...interface{}) {
	z.Logf(log.FATAL, msg, params...)
}

func (z *ZapLoggerAdapter) Panic(i ...interface{}) {
	z.Log(log.PANIC, i...)
}

func (z *ZapLoggerAdapter) Panicf(msg string, params ...interface{}) {
	z.Logf(log.PANIC, msg, params...)
}

func (z *ZapLoggerAdapter) Child(name string) log.Logger {
	l := z.Named(name)
	return &ZapLoggerAdapter{Logger: l}
}

func (z *ZapLoggerAdapter) WithMeta(key string, value interface{}) log.Logger {
	l := z.Logger.With(zap.Any(key, value))
	return &ZapLoggerAdapter{Logger: l, lvl: z.lvl}
}

func (z *ZapLoggerAdapter) GetLevel() log.Level {
	return z.lvl
}
