// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package observability

import (
	"context"
	"net/http"

	"contrib.go.opencensus.io/exporter/prometheus"
	enseada "github.com/enseadaio/enseada/pkg"
	"github.com/labstack/echo"
	"go.opencensus.io/plugin/ochttp"
	"go.opencensus.io/plugin/runmetrics"
)

func Boot(ctx context.Context, e *echo.Echo) (enseada.StopFunc, error) {
	rep, err := prometheus.NewExporter(prometheus.Options{
		Namespace: "enseada",
		OnError:   nil, // TODO: error reporting
	})
	if err != nil {
		return nil, err
	}

	if err := runmetrics.Enable(runmetrics.RunMetricOptions{
		EnableCPU:    true,
		EnableMemory: true,
		Prefix:       "runtime_",
	}); err != nil {
		return nil, err
	}

	e.Pre(echo.WrapMiddleware(func(base http.Handler) http.Handler {
		return &ochttp.Handler{
			Handler: base,
			IsHealthEndpoint: func(r *http.Request) bool {
				return r.URL.Path == "/health"
			},
		}
	}))
	e.GET("/metrics", echo.WrapHandler(rep))
	return func(sctx context.Context) error {
		runmetrics.Disable()
		return nil
	}, nil
}
