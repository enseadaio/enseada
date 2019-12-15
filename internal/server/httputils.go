package server

import (
	"fmt"
	"net/http"
	"strings"
)

type HTTPErrorBody struct {
	Error   string `json:"error"`
	Message string `json:"message"`
}

func HTTPError(status int, format string, args ...interface{}) HTTPErrorBody {
	err := http.StatusText(status)
	err = strings.ToLower(err)
	err = strings.ReplaceAll(err, " ", "_")
	msg := format
	if len(args) > 0 {
		msg = fmt.Sprintf(format, args)
	}

	return HTTPErrorBody{
		Error:   err,
		Message: msg,
	}
}
