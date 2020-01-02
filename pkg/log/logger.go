// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package log

type Logger interface {
	Log(lvl Level, i ...interface{})
	Logf(lvl Level, msg string, params ...interface{})
	Trace(i ...interface{})
	Tracef(msg string, params ...interface{})
	Debug(i ...interface{})
	Debugf(msg string, params ...interface{})
	Info(i ...interface{})
	Infof(msg string, params ...interface{})
	Warn(i ...interface{})
	Warnf(msg string, params ...interface{})
	Error(i ...interface{})
	Errorf(msg string, params ...interface{})
	Fatal(i ...interface{})
	Fatalf(msg string, params ...interface{})
	Panic(i ...interface{})
	Panicf(msg string, params ...interface{})
	Child(name string) Logger
	WithMeta(key string, value interface{}) Logger
	GetLevel() Level
}
