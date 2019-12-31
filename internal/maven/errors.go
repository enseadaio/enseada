// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"errors"
	"fmt"
	"github.com/twitchtv/twirp"
)

func formatError(format string, args ...interface{}) error {
	return errors.New(fmt.Sprintf(format, args...))
}

var (
	ErrorRepoAlreadyPresent = errors.New("repository already present")
	ErrorRepoNotFound       = errors.New("repository not found")
	ErrorTooManyFilesForKey = func(expected, actual int) error {
		return formatError("too many files found. Expected %d, found %d", expected, actual)
	}
	ErrorInvalidRepoId = func(id string) error {
		return formatError("invalid repo id. %s is not a valid Maven repo identifier", id)
	}
)

var (
	TwirpRepoNotFoundError = func(id string) twirp.Error {
		return twirp.NotFoundError(fmt.Sprintf("no Maven repository found by id %s", id))
	}
)
