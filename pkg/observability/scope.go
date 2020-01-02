// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package observability

import (
	"io"
	"time"

	"github.com/enseadaio/enseada/pkg/log"
	"github.com/uber-go/tally"
	tallyprom "github.com/uber-go/tally/prometheus"
)

func NewPromReporter(l log.Logger) tallyprom.Reporter {
	return tallyprom.NewReporter(tallyprom.Options{
		OnRegisterError: func(err error) {
			l.Fatal(err)
		},
	})
}

func NewScope(r tally.CachedStatsReporter) (tally.Scope, io.Closer) {
	return tally.NewRootScope(tally.ScopeOptions{
		Prefix:         "enseada",
		Tags:           map[string]string{},
		CachedReporter: r,
		Separator:      tallyprom.DefaultSeparator,
	}, time.Second)
}
