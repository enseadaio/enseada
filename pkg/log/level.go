// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package log

type Level string

const (
	TRACE = Level("trace")
	DEBUG = Level("debug")
	INFO  = Level("info")
	WARN  = Level("warn")
	ERROR = Level("error")
	FATAL = Level("fatal")
	PANIC = Level("panic")
)
