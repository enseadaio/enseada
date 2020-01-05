// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package app

import "context"

type LifecycleEvent uint8

const (
	BeforeApplicationStartEvent LifecycleEvent = iota
	AfterApplicationStartEvent
	BeforeApplicationStopEvent
	AfterApplicationStopEvent
)

type OnLifecycleEventFunc func(ctx context.Context, event LifecycleEvent) error
