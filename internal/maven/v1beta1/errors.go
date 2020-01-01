package mavenv1beta1api

import (
	"fmt"
	"github.com/twitchtv/twirp"
)

var (
	TwirpRepoNotFoundError = func(id string) twirp.Error {
		return twirp.NotFoundError(fmt.Sprintf("no Maven repository found by id %s", id))
	}
)
