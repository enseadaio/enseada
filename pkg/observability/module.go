// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package observability

import (
	"context"
	"net/http"

	"github.com/enseadaio/enseada/pkg/errare"
	"github.com/enseadaio/enseada/pkg/log"
	"go.opencensus.io/stats/view"

	"contrib.go.opencensus.io/exporter/prometheus"
	"github.com/labstack/echo"
	"go.opencensus.io/plugin/ochttp"
	"go.opencensus.io/plugin/runmetrics"
)

type Module struct {
	logger log.Logger
}

func NewModule(logger log.Logger, e *echo.Echo, errh errare.Handler) (*Module, error) {
	rep, err := prometheus.NewExporter(prometheus.Options{
		Namespace: "enseada",
		OnError:   errh.HandleError,
	})
	if err != nil {
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

	if err := runmetrics.Enable(runmetrics.RunMetricOptions{
		EnableCPU:    true,
		EnableMemory: true,
		Prefix:       "runtime_",
	}); err != nil {
		return nil, err
	}

	if err := view.Register(
		ochttp.ServerRequestCountView,
		ochttp.ServerRequestBytesView,
		ochttp.ServerResponseBytesView,
		ochttp.ServerLatencyView,
		ochttp.ServerRequestCountByMethod,
		ochttp.ServerResponseCountByStatusCode,
	); err != nil {
		return nil, err
	}

	return &Module{logger: logger}, nil
}

func (m *Module) Start(ctx context.Context) error {
	m.logger.Info("started observability module")
	return nil
}

func (m *Module) Stop(ctx context.Context) error {
	runmetrics.Disable()
	m.logger.Info("stopped observability module")
	return nil
}
