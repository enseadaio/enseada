// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package server

import (
	"fmt"
	"github.com/labstack/echo"
	"net/http"
	"strings"
)

type HTTPErrorBody struct {
	Error   string `json:"error"`
	Message string `json:"message"`
}

func HTTPError(status int, format string, args ...interface{}) HTTPErrorBody {
	err := http.StatusText(status)
	err = strings.ToLower(err)
	err = strings.ReplaceAll(err, " ", "_")
	msg := format
	if len(args) > 0 {
		msg = fmt.Sprintf(format, args...)
	}

	return HTTPErrorBody{
		Error:   err,
		Message: msg,
	}
}

func QueryWithDefault(c echo.Context, name string, def string) string {
	p := c.QueryParam(name)
	if p == "" {
		return def
	}

	return p
}
