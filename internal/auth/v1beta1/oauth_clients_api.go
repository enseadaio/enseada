package authv1beta1api

import (
	"context"
	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/internal/middleware"
	"github.com/enseadaio/enseada/internal/scope"
	"github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
	"github.com/ory/fosite"
	"github.com/twitchtv/twirp"
)

type OAuthClientsService struct {
	Logger   echo.Logger
	Enforcer *casbin.Enforcer
	Store    *auth.Store
}

func (o *OAuthClientsService) ListClients(ctx context.Context, req *authv1beta1.ListClientsRequest) (*authv1beta1.ListClientsResponse, error) {
	id, ok := middleware.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := middleware.Scopes(ctx)
	if !scopes.Has(scope.OAuthReadClients) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	var cs []fosite.Client
	if id == "root" {
		clients, err := o.Store.ListClients(ctx, couch.Query{})
		if err != nil {
			return nil, err
		}

		cs = clients
	} else {
		ps := o.Enforcer.GetPermissionsForUser(id)
		ids := make([]string, 0)
		for _, p := range ps {
			g, err := guid.Parse(p[1])
			if err != nil {
				return nil, err
			}

			if g.Kind() == couch.KindOAuthClient && p[2] == "read" {
				ids = append(ids, g.ID())
			}
		}

		clients, err := o.Store.ListClients(ctx, couch.Query{
			"_id": couch.Query{
				"$in": ids,
			},
		})
		if err != nil {
			return nil, err
		}

		cs = clients
	}

	var clients []*authv1beta1.OAuthClient

	for _, c := range cs {
		clients = append(clients, mapClientToProto(c))
	}

	return &authv1beta1.ListClientsResponse{
		Clients: clients,
	}, nil
}

func (o *OAuthClientsService) GetClient(ctx context.Context, req *authv1beta1.GetClientRequest) (*authv1beta1.GetClientResponse, error) {
	id, ok := middleware.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := middleware.Scopes(ctx)
	if !scopes.Has(scope.OAuthReadClients) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.OAuthDB, req.Id, couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg.String(), "read")
	if err != nil {
		return nil, err
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	c, err := o.Store.GetClient(ctx, req.Id)
	if err != nil {
		return nil, err
	}

	if c == nil {
		return nil, twirp.NotFoundError(req.Id)
	}

	return &authv1beta1.GetClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func (o *OAuthClientsService) CreateClient(ctx context.Context, req *authv1beta1.CreateClientRequest) (*authv1beta1.CreateClientResponse, error) {
	id, ok := middleware.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := middleware.Scopes(ctx)
	if !scopes.Has(scope.OAuthWriteClients) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	pc := req.GetClient()
	if pc == nil {
		return nil, twirp.RequiredArgumentError("client")
	}

	if pc.GetPublic() && req.GetSecret() == "" {
		return nil, twirp.RequiredArgumentError("client secret")
	}

	c, err := mapProtoToClient(pc, req.GetSecret())
	if err != nil {
		return nil, err
	}

	err = o.Store.SaveClient(ctx, c)
	if err != nil {
		if kivik.StatusCode(err) == kivik.StatusConflict {
			return nil, twirp.NewError(twirp.AlreadyExists, "")
		}
		return nil, err
	}

	cg := guid.New(couch.OAuthDB, c.GetID(), couch.KindOAuthClient)
	ps := []string{"read", "update", "delete"}
	for _, p := range ps {
		_, err := o.Enforcer.AddPermissionForUser(id, cg.String(), p)
		if err != nil {
			return nil, err
		}
	}

	return &authv1beta1.CreateClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func (o *OAuthClientsService) UpdateClient(ctx context.Context, req *authv1beta1.UpdateClientRequest) (*authv1beta1.UpdateClientResponse, error) {
	id, ok := middleware.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := middleware.Scopes(ctx)
	if !scopes.Has(scope.OAuthWriteClients) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	pc := req.GetClient()
	if pc == nil {
		return nil, twirp.RequiredArgumentError("client")
	}

	fc, err := o.Store.GetClient(ctx, pc.GetId())
	if err != nil {
		return nil, err
	}

	if fc == nil {
		return nil, twirp.NotFoundError("")
	}

	c := fc.(*auth.OAuthClient)

	cg := guid.New(couch.OAuthDB, c.GetID(), couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg.String(), "update")
	if err != nil {
		return nil, err
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	if pc.GetRedirectUris() != nil {
		c.RedirectURIs = pc.GetRedirectUris()
	}
	if pc.GetGrantTypes() != nil {
		c.GrantTypes = pc.GetGrantTypes()
	}
	if pc.GetResponseTypes() != nil {
		c.ResponseTypes = pc.GetResponseTypes()
	}
	if pc.GetScopes() != nil {
		c.Scopes = pc.GetScopes()
	}
	if pc.GetAudiences() != nil {
		c.Audiences = pc.GetAudiences()
	}

	err = o.Store.SaveClient(ctx, c)
	return &authv1beta1.UpdateClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func (o *OAuthClientsService) DeleteClient(ctx context.Context, req *authv1beta1.DeleteClientRequest) (*authv1beta1.DeleteClientResponse, error) {
	id, ok := middleware.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := middleware.Scopes(ctx)
	if !scopes.Has(scope.OAuthWriteClients) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.OAuthDB, req.GetId(), couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg, "delete")
	if err != nil {
		return nil, err
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	c, err := o.Store.DeleteClient(ctx, req.GetId())
	if err != nil {
		return nil, err
	}

	if c == nil {
		return nil, twirp.NotFoundError("")
	}

	return &authv1beta1.DeleteClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func mapClientToProto(c fosite.Client) *authv1beta1.OAuthClient {
	return &authv1beta1.OAuthClient{
		Id:            c.GetID(),
		RedirectUris:  c.GetRedirectURIs(),
		GrantTypes:    c.GetGrantTypes(),
		ResponseTypes: c.GetResponseTypes(),
		Scopes:        c.GetScopes(),
		Audiences:     c.GetAudience(),
		Public:        c.IsPublic(),
	}
}

func mapProtoToClient(c *authv1beta1.OAuthClient, secret string) (fosite.Client, error) {
	return auth.NewOAuthClient(c.GetId(), secret,
		auth.OAuthRedirectURIs(c.GetRedirectUris()...),
		auth.OAuthGrantTypes(c.GetGrantTypes()...),
		auth.OAuthResponseTypes(c.GetResponseTypes()...),
		auth.OAuthScopes(c.GetScopes()...),
		auth.OAuthAudiences(c.GetAudiences()...),
		auth.OAuthPublic(c.GetPublic()),
	)
}
