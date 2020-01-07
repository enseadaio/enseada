// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"errors"
	"fmt"
)

var (
	ErrRepoAlreadyPresent = errors.New("repository already present")
	ErrRepoNotFound       = errors.New("repository not found")
	ErrTooManyFilesForKey = func(expected, actual int) error {
		return fmt.Errorf("too many files found. Expected %d, found %d", expected, actual)
	}
	ErrInvalidRepoId = func(id string) error {
		return fmt.Errorf("invalid repo id. %s is not a valid Maven repo identifier", id)
	}
	ErrImmutableVersion = func(v string) error {
		return fmt.Errorf("version %v is immutable. Deploy a new version or use a SNAPSHOT qualifier", v)
	}
)
