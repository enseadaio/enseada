// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package middleware

import (
	"net/http"

	"github.com/uber-go/tally"
)

func Stats(stats tally.Scope) func(http.Handler) http.Handler {
	requests := stats.Histogram("requests_duration", tally.DefaultBuckets)
	return func(base http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.RequestURI == "/metrics" {
				base.ServeHTTP(w, r)
				return
			}

			watch := requests.Start()
			base.ServeHTTP(w, r)
			watch.Stop()
		})
	}
}
