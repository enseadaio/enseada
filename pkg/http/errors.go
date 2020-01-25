// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package http

import (
	"net/http"
	"strings"

	"github.com/enseadaio/enseada/internal/utils"
	"github.com/enseadaio/enseada/pkg/errare"
	session "github.com/ipfans/echo-session"
	"github.com/labstack/echo"
	"github.com/pkg/errors"
)

type statusCoder interface {
	StatusCode() int
}

type causer interface {
	Cause() error
}

func newErrorHandler(errh errare.Handler) echo.HTTPErrorHandler {
	return func(err error, c echo.Context) {
		errh.HandleError(err)

		if he, ok := err.(*echo.HTTPError); ok {
			var msg string
			if he.Internal != nil {
				msg = he.Internal.Error()
			} else if m, ok := he.Message.(string); ok {
				msg = m
			} else {
				msg = he.Error()
			}

			s := session.Default(c)
			if strings.Contains(c.Request().Header.Get("accept"), "html") && s != nil {
				s.AddFlash(msg, "HTTPError")
				if err := s.Save(); err != nil {
					errh.HandleError(err)
					return
				}

				if e := c.Redirect(http.StatusTemporaryRedirect, "/ui/error"); e != nil {
					errh.HandleError(err)
				}
				return
			}

			if e := c.JSON(he.Code, utils.HTTPError(he.Code, msg)); e != nil {
				errh.HandleError(err)
			}
			return
		}

		sc := http.StatusInternalServerError
		var coder statusCoder
		for {
			if errors.As(err, &coder) {
				sc = coder.StatusCode()
				break
			}
			if uw := errors.Unwrap(err); uw != nil {
				err = uw
				continue
			}

			if c, ok := err.(causer); ok {
				err = c.Cause()
				continue
			}
			break
		}

		e := c.JSON(sc, utils.HTTPError(sc, err.Error()))
		if e != nil {
			errh.HandleError(err)
		}
	}
}
