// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"encoding/json"
	"io"
	"io/ioutil"
	"net/http"

	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/uber-go/tally"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/utils"
	"github.com/labstack/echo"
	eware "github.com/labstack/echo/middleware"
	elog "github.com/labstack/gommon/log"
)

func createEchoServer(l log.Logger, stats tally.Scope) *echo.Echo {
	e := echo.New()

	e.Logger = &EchoLogger{
		Logger: l,
		pfx:    "echo",
	}
	e.HideBanner = true
	e.HTTPErrorHandler = func(err error, c echo.Context) {
		e := c.JSON(http.StatusInternalServerError, utils.HTTPError(http.StatusInternalServerError, err.Error()))
		if e != nil {
			c.Logger().Error(e)
		}
	}

	e.Renderer = NewGoViewRenderer()

	e.Use(eware.Recover())
	e.Use(eware.CORS())
	e.Use(eware.RequestID())
	e.Use(eware.Logger())
	e.Use(eware.GzipWithConfig(eware.GzipConfig{
		Level: 5,
		Skipper: func(c echo.Context) bool {
			return c.Path() == "/metrics"
		},
	}))
	e.Pre(eware.RemoveTrailingSlashWithConfig(
		eware.TrailingSlashConfig{
			RedirectCode: http.StatusMovedPermanently,
		}))
	e.Use(echo.WrapMiddleware(middleware.Stats(stats)))

	return e
}

type EchoLogger struct {
	log.Logger
	pfx string
}

func (e *EchoLogger) Output() io.Writer {
	return ioutil.Discard // not supported ATM
}

func (e *EchoLogger) SetOutput(w io.Writer) {
	// not supported ATM
}

func (e *EchoLogger) Prefix() string {
	return e.pfx
}

func (e *EchoLogger) SetPrefix(p string) {
	e = e.Child(p).(*EchoLogger)
	e.pfx = p
}

func (e *EchoLogger) Level() elog.Lvl {
	switch e.GetLevel() {
	case log.TRACE:
		return elog.DEBUG
	case log.DEBUG:
		return elog.DEBUG
	case log.INFO:
		return elog.INFO
	case log.WARN:
		return elog.WARN
	case log.ERROR:
		return elog.ERROR
	case log.FATAL:
		return elog.ERROR
	default:
		return elog.INFO
	}
}

func (e *EchoLogger) SetLevel(v elog.Lvl) {
	// not supported ATM
}

func (e *EchoLogger) SetHeader(h string) {
	// not supported ATM
}

func (e *EchoLogger) Print(i ...interface{}) {
	e.Trace(i)
}

func (e *EchoLogger) Printf(format string, args ...interface{}) {
	e.Tracef(format, args)
}

func (e *EchoLogger) Printj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Trace(b)
}

func (e *EchoLogger) Debugj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Debug(b)
}

func (e *EchoLogger) Infoj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Info(b)
}

func (e *EchoLogger) Warnj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Warn(b)
}

func (e *EchoLogger) Errorj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Error(b)
}

func (e *EchoLogger) Fatalj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Fatal(b)
}

func (e *EchoLogger) Panicj(j elog.JSON) {
	b, err := json.Marshal(j)
	if err != nil {
		e.Panic(err)
	}
	e.Panic(b)
}
