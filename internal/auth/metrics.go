// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"

	"go.opencensus.io/metric"
	"go.opencensus.io/metric/metricdata"
)

type MetricsRegistry interface {
	UsersCount() *metric.Int64GaugeEntry
}

type Metrics struct {
	usersCount *metric.Int64GaugeEntry
}

func (m *Metrics) UsersCount() *metric.Int64GaugeEntry {
	return m.usersCount
}

func InitMetrics(ctx context.Context, r *metric.Registry, s *Store) (*Metrics, error) {
	ug, err := r.AddInt64Gauge(
		"users_count",
		metric.WithDescription("The number of registered users"),
		metric.WithUnit(metricdata.UnitDimensionless),
	)
	if err != nil {
		return nil, err
	}

	uc, err := ug.GetEntry()
	if err != nil {
		return nil, err
	}

	us, err := s.ListUsers(ctx)
	if err != nil {
		return nil, err
	}
	uc.Set(int64(len(us)))

	return &Metrics{
		usersCount: uc,
	}, nil
}
