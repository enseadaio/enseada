package enseada

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"github.com/casbin/casbin/v2"
	"github.com/chartmuseum/storage"
	"github.com/enseadaio/enseada/internal/auth"
	authsvcv1beta1 "github.com/enseadaio/enseada/internal/authsvc/v1beta1"
	"github.com/enseadaio/enseada/internal/maven"
	mavensvcv1beta1 "github.com/enseadaio/enseada/internal/mavensvc/v1beta1"
	"github.com/enseadaio/enseada/internal/server"
	"github.com/enseadaio/enseada/internal/users"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/labstack/echo/middleware"
	"github.com/labstack/gommon/log"
	"github.com/ory/fosite"
	"github.com/ory/fosite/compose"
	goauth "golang.org/x/oauth2"
	"net/http"
)

type Server struct {
	*echo.Echo
	Maven         *maven.Maven
	Enforcer      *casbin.Enforcer
	UserSvc       *users.UserSvc
	OAuthServer   fosite.OAuth2Provider
	OAuthClient   goauth.Config
	Users         *users.UserSvc
	SecretKeyBase []byte
	PublicHost    string
}

func handleErrors(err error, c echo.Context) {
	e := c.JSON(http.StatusInternalServerError, server.HTTPError(http.StatusInternalServerError, err.Error()))
	if e != nil {
		c.Logger().Error(e)
	}
}

func NewServer(client *kivik.Client, storage storage.Backend, en *casbin.Enforcer, u *users.UserSvc, opts ...ServerOption) (*Server, error) {
	options := &ServerOptions{
		level:      log.INFO,
		publicHost: "localhost:9623",
	}

	for _, opt := range opts {
		opt(options)
	}

	e := echo.New()

	e.Logger.SetLevel(options.level)
	e.HideBanner = true
	e.HTTPErrorHandler = handleErrors
	e.Renderer = server.NewGoViewRenderer()

	e.Use(middleware.Recover())
	e.Use(middleware.CORS())
	e.Use(middleware.RequestID())
	e.Use(middleware.Logger())
	e.Use(middleware.GzipWithConfig(middleware.GzipConfig{
		Level: 5,
	}))
	e.Pre(middleware.RemoveTrailingSlashWithConfig(
		middleware.TrailingSlashConfig{
			RedirectCode: http.StatusMovedPermanently,
		}))

	mvnLog := log.New("maven2")
	mvnLog.SetLevel(options.level)
	mvn, err := maven.New(client, storage, mvnLog)
	if err != nil {
		return nil, err
	}

	oauthLog := log.New("oauth2")
	config := new(compose.Config)
	store, err := auth.NewStore(client, oauthLog)
	if err != nil {
		return nil, err
	}

	key, err := rsa.GenerateKey(rand.Reader, 1024)
	if err != nil {
		return nil, err
	}

	secretKeyBase := options.secretKeyBase
	if secretKeyBase == "" {
		panic("No secret key base provided")
	}

	if len(secretKeyBase) < 32 {
		panic("Secret key base must be longer than 32 bytes")
	}

	skb := []byte(secretKeyBase)

	oauth2 := compose.ComposeAllEnabled(
		config,
		store,
		skb,
		key,
	)

	secret := options.defaultOauthClientSecret
	if secret == "" {
		panic("No default OAuthServer client secret provided")
	}

	publicHost := options.publicHost
	if publicHost == "" {
		panic("No public host provided")
	}

	err = store.InitDefaultClient(context.Background(), publicHost, secret)
	if err != nil {
		return nil, err
	}

	oc := goauth.Config{
		ClientID:     "enseada",
		ClientSecret: secret,
		Endpoint: goauth.Endpoint{
			AuthURL:   publicHost + "/oauth/authorize",
			TokenURL:  publicHost + "/oauth/token",
			AuthStyle: goauth.AuthStyleAutoDetect,
		},
		RedirectURL: publicHost + "/ui/callback",
		Scopes:      []string{"openid"},
	}

	return &Server{
		Echo:          e,
		Maven:         mvn,
		Enforcer:      en,
		UserSvc:       u,
		OAuthServer:   oauth2,
		OAuthClient:   oc,
		Users:         u,
		PublicHost:    publicHost,
		SecretKeyBase: skb,
	}, nil
}

func (s *Server) Init() {
	mvnsvc := mavensvcv1beta1.Service{Maven: s.Maven}
	mvnHandler := mavenv1beta1.NewMavenAPIServer(mvnsvc, nil)
	s.Echo.Any(mvnHandler.PathPrefix()+"*", echo.WrapHandler(mvnHandler))

	authLog := log.New("casbin")
	authLog.SetLevel(s.Logger.Level())
	authsvc := authsvcv1beta1.Service{
		Logger:   authLog,
		Enforcer: s.Enforcer,
	}
	authHandler := authv1beta1.NewAclAPIServer(authsvc, nil)
	s.Echo.Any(authHandler.PathPrefix()+"*", echo.WrapHandler(authHandler))

	server.MountRoutes(server.RouteParams{
		Echo:          s.Echo,
		Mvn:           s.Maven,
		UserSvc:       s.UserSvc,
		OAuthProvider: s.OAuthServer,
		OAuthClient:   s.OAuthClient,
		PublicHost:    s.PublicHost,
		SecretKeyBase: s.SecretKeyBase,
	})
}
