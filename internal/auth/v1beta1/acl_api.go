// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"

	"github.com/ory/fosite"

	"github.com/enseadaio/enseada/pkg/log"

	"github.com/enseadaio/enseada/internal/ctxutils"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/internal/scope"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/twitchtv/twirp"
)

type AclAPI struct {
	Logger   log.Logger
	Enforcer *casbin.Enforcer
}

func NewAclAPI(logger log.Logger, enforcer *casbin.Enforcer) *AclAPI {
	return &AclAPI{Logger: logger, Enforcer: enforcer}
}

func (s *AclAPI) ListPermissions(ctx context.Context, req *authv1beta1.ListPermissionsRequest) (*authv1beta1.ListPermissionsResponse, error) {
	_, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "unauthenticated")
	}
	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.ACLPermissionRead) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	policy := s.Enforcer.GetPolicy()
	var permissions []*authv1beta1.AclPermission

	for _, r := range policy {
		var permission authv1beta1.AclPermission
		if len(r) > 0 {
			permission.Sub = r[0]
		}
		if len(r) > 1 {
			permission.Obj = r[1]
		}
		if len(r) > 2 {
			permission.Act = r[2]
		}
		permissions = append(permissions, &permission)
	}
	if len(permissions) == 0 {
		return nil, twirp.NotFoundError("no permissions found")
	}

	return &authv1beta1.ListPermissionsResponse{
		Permissions: permissions,
	}, nil
}

func (s *AclAPI) AddPermission(ctx context.Context, req *authv1beta1.AddPermissionRequest) (*authv1beta1.AddPermissionResponse, error) {
	scopes, ok := ctxutils.Scopes(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "unauthenticated")
	}

	if !fosite.WildcardScopeStrategy(scopes, scope.ACLPermissionWrite) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	permission := req.Permission
	if permission == nil {
		return nil, twirp.RequiredArgumentError("permission")
	}

	if _, err := guid.Parse(permission.Obj); err != nil {
		return nil, twirp.InvalidArgumentError("obj", err.Error())
	}

	if permission.Act == "" {
		return nil, twirp.RequiredArgumentError("act")
	}

	ok, err := s.Enforcer.AddPolicy(permission.Sub, permission.Obj, permission.Act)
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if ok {
		return &authv1beta1.AddPermissionResponse{Permission: permission}, nil
	}

	return nil, twirp.NewError(twirp.AlreadyExists, "permission already exists")
}

func (s *AclAPI) DeletePermission(ctx context.Context, req *authv1beta1.DeletePermissionRequest) (*authv1beta1.DeletePermissionResponse, error) {
	scopes, ok := ctxutils.Scopes(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "unauthenticated")
	}

	if !fosite.WildcardScopeStrategy(scopes, scope.ACLPermissionDelete) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	perm := req.Permission
	if perm == nil {
		return nil, twirp.RequiredArgumentError("perm")
	}

	if _, err := guid.Parse(perm.Obj); err != nil {
		return nil, twirp.InvalidArgumentError("sub", err.Error())
	}

	if perm.Act == "" {
		return nil, twirp.RequiredArgumentError("act")
	}

	ok, err := s.Enforcer.RemovePolicy(perm.Sub, perm.Obj, perm.Act)
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if ok {
		return &authv1beta1.DeletePermissionResponse{Permission: perm}, nil
	}

	return nil, twirp.NotFoundError("perm not found")
}
