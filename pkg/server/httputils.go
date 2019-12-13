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
	return HTTPErrorBody{
		Error:   err,
		Message: fmt.Sprintf(format, args),
	}
}
