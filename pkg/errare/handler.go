// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package errare

import (
	"context"
	"io"
)

type Handler interface {
	io.Closer
	HandleError(err error)
	HandlePanic(v interface{})
	HandlePanicWithContext(ctx context.Context, v interface{})
	SetCurrentUser(id string, extras Extras)
}
