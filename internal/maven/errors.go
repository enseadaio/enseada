package maven

import (
	"errors"
	"fmt"
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
