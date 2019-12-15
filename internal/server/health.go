package server

import (
	"github.com/labstack/echo"
	"net/http"
)

type HealthCheckResponse struct {
	Status   string `json:"status"`
	Protocol string `json:"protocol"`
	Host     string `json:"host"`
	Remote   string `json:"remote"`
	Method   string `json:"method"`
	Path     string `json:"path"`
}

func mountHealthCheck(e *echo.Echo) {
	e.GET("/health", func(c echo.Context) error {
		req := c.Request()
		res := HealthCheckResponse{
			Status:   "UP",
			Protocol: req.Proto,
			Host:     req.Host,
			Remote:   req.RemoteAddr,
			Method:   req.Method,
			Path:     req.URL.Path,
		}
		return c.JSON(http.StatusOK, res)
	})
}
