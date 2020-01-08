// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/ctxutils"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/internal/scope"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/ory/fosite"
	"github.com/twitchtv/twirp"
)

type OAuthClientsAPI struct {
	Logger   log.Logger
	Enforcer *casbin.Enforcer
	Store    *auth.Store
}

func NewOAuthClientsAPI(logger log.Logger, enforcer *casbin.Enforcer, store *auth.Store) *OAuthClientsAPI {
	return &OAuthClientsAPI{Logger: logger, Enforcer: enforcer, Store: store}
}

func (o *OAuthClientsAPI) ListClients(ctx context.Context, req *authv1beta1.ListClientsRequest) (*authv1beta1.ListClientsResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.OAuthClientRead) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	var cs []fosite.Client
	if id == "root" {
		clients, err := o.Store.ListClients(ctx, couch.Query{})
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}

		cs = clients
	} else {
		ps := o.Enforcer.GetPermissionsForUser(id)
		ids := make([]string, 0)
		for _, p := range ps {
			g, err := guid.Parse(p[1])
			if err != nil {
				return nil, twirp.InternalErrorWith(err)
			}

			if g.DB() == couch.OAuthDB && g.Kind() == couch.KindOAuthClient && p[2] == "read" {
				ids = append(ids, g.ID())
			}
		}

		clients, err := o.Store.ListClients(ctx, couch.Query{
			"_id": couch.Query{
				"$in": ids,
			},
		})
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
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

func (o *OAuthClientsAPI) GetClient(ctx context.Context, req *authv1beta1.GetClientRequest) (*authv1beta1.GetClientResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.OAuthClientRead) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.OAuthDB, req.Id, couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg.String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	c, err := o.Store.GetClient(ctx, req.Id)
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if c == nil {
		return nil, twirp.NotFoundError(req.Id)
	}

	return &authv1beta1.GetClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func (o *OAuthClientsAPI) CreateClient(ctx context.Context, req *authv1beta1.CreateClientRequest) (*authv1beta1.CreateClientResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.OAuthClientWrite) {
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
		return nil, twirp.InternalErrorWith(err)
	}

	err = o.Store.SaveClient(ctx, c)
	if err != nil {
		if kivik.StatusCode(err) == kivik.StatusConflict {
			return nil, twirp.NewError(twirp.AlreadyExists, "")
		}
		return nil, twirp.InternalErrorWith(err)
	}

	cg := guid.New(couch.OAuthDB, c.GetID(), couch.KindOAuthClient)
	ps := []string{"read", "update", "delete"}
	for _, p := range ps {
		_, err := o.Enforcer.AddPermissionForUser(id, cg.String(), p)
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}
	}

	return &authv1beta1.CreateClientResponse{
		Client: mapClientToProto(c),
	}, nil
}

func (o *OAuthClientsAPI) UpdateClient(ctx context.Context, req *authv1beta1.UpdateClientRequest) (*authv1beta1.UpdateClientResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.OAuthClientWrite) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	pc := req.GetClient()
	if pc == nil {
		return nil, twirp.RequiredArgumentError("client")
	}

	fc, err := o.Store.GetClient(ctx, pc.GetId())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if fc == nil {
		return nil, twirp.NotFoundError("")
	}

	c := fc.(*auth.OAuthClient)

	cg := guid.New(couch.OAuthDB, c.GetID(), couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg.String(), "update")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
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

func (o *OAuthClientsAPI) DeleteClient(ctx context.Context, req *authv1beta1.DeleteClientRequest) (*authv1beta1.DeleteClientResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.OAuthClientWrite) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.OAuthDB, req.GetId(), couch.KindOAuthClient)
	can, err := o.Enforcer.Enforce(id, cg, "delete")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	c, err := o.Store.DeleteClient(ctx, req.GetId())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if c == nil {
		return nil, twirp.NotFoundError("")
	}

	ps := []string{"read", "update", "delete"}
	for _, p := range ps {
		_, err := o.Enforcer.DeletePermissionForUser(id, cg.String(), p)
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}
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
