// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"

	"github.com/enseadaio/enseada/internal/scope"
	"github.com/ory/fosite"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/ctxutils"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/pkg/log"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/twitchtv/twirp"
)

var (
	ErrUnauthenticated         = twirp.NewError(twirp.Unauthenticated, "unauthenticated")
	ErrInsufficientPermissions = twirp.NewError(twirp.PermissionDenied, "insufficient permissions")
	ErrNoUsersFound            = twirp.NotFoundError("no users found")
)

type UsersAPI struct {
	Logger   log.Logger
	Enforcer *casbin.Enforcer
	Store    auth.UserStorage
	m        auth.MetricsRegistry
}

func NewUsersAPI(logger log.Logger, enforcer *casbin.Enforcer, s auth.UserStorage, m auth.MetricsRegistry) *UsersAPI {
	return &UsersAPI{Logger: logger, Enforcer: enforcer, Store: s, m: m}
}

func (u *UsersAPI) ListUsers(ctx context.Context, req *authv1beta1.ListUsersRequest) (*authv1beta1.ListUsersResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	can, err := u.Enforcer.Enforce(uid, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, ErrInsufficientPermissions
	}

	users, err := u.Store.ListUsers(ctx)
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	usc := len(users)
	if usc == 0 {
		return nil, ErrNoUsersFound
	}

	us := make([]*authv1beta1.User, usc)
	for i, u := range users {
		up := &authv1beta1.User{
			Username: u.Username,
		}
		us[i] = up
	}

	u.m.UsersCount().Set(int64(usc))
	return &authv1beta1.ListUsersResponse{
		Users: us,
	}, nil
}

func (u *UsersAPI) GetUser(ctx context.Context, req *authv1beta1.GetUserRequest) (*authv1beta1.GetUserResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	can, err := u.Enforcer.Enforce(uid, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, ErrInsufficientPermissions
	}

	if req.GetUsername() == "" {
		return nil, twirp.RequiredArgumentError("username")
	}

	user, err := u.Store.GetUser(ctx, req.GetUsername())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if user == nil {
		return nil, twirp.NotFoundError("user not found")
	}

	up := &authv1beta1.User{
		Username: user.Username,
	}
	return &authv1beta1.GetUserResponse{
		User: up,
	}, nil
}

func (u *UsersAPI) GetCurrentUser(ctx context.Context, req *authv1beta1.GetCurrentUserRequest) (*authv1beta1.GetCurrentUserResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !fosite.WildcardScopeStrategy(scopes, scope.Profile) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	uu, err := u.Store.GetUser(ctx, uid)
	if err != nil {
		return nil, err
	}

	if uu == nil {
		return nil, twirp.NotFoundError("no user found with id " + uid)
	}

	return &authv1beta1.GetCurrentUserResponse{
		User: &authv1beta1.User{
			Username: uu.Username,
		},
	}, nil
}

func (u *UsersAPI) CreateUser(ctx context.Context, req *authv1beta1.CreateUserRequest) (*authv1beta1.CreateUserResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	can, err := u.Enforcer.Enforce(uid, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "write")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, ErrInsufficientPermissions
	}

	up := req.GetUser()
	if up == nil {
		return nil, twirp.RequiredArgumentError("user")
	}

	pwd := req.GetPassword()
	if pwd == "" {
		return nil, twirp.RequiredArgumentError("password")
	}

	uu := &auth.User{
		Username: up.GetUsername(),
		Password: pwd,
	}
	err = u.Store.CreateUser(ctx, uu)
	if err != nil {
		// Don't like it, leaking db implementation
		if kivik.StatusCode(err) == kivik.StatusConflict {
			e := twirp.NewError(twirp.AlreadyExists, "user already exists")
			e = e.WithMeta("username", up.GetUsername())
			return nil, e
		}
		return nil, twirp.InternalErrorWith(err)
	}

	u.m.UsersCount().Add(1)
	return &authv1beta1.CreateUserResponse{
		User: up,
	}, nil
}

func (u *UsersAPI) UpdateUserPassword(ctx context.Context, req *authv1beta1.UpdateUserPasswordRequest) (*authv1beta1.UpdateUserPasswordResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	uu, err := u.Store.GetUser(ctx, uid)
	if err != nil {
		return nil, err
	}

	if req.GetPassword() == "" {
		return nil, twirp.RequiredArgumentError("password")
	}

	uu.Password = req.GetPassword()
	err = u.Store.UpdateUser(ctx, uu)
	if err != nil {
		return nil, err
	}

	return &authv1beta1.UpdateUserPasswordResponse{}, nil
}

func (u *UsersAPI) DeleteUser(ctx context.Context, req *authv1beta1.DeleteUserRequest) (*authv1beta1.DeleteUserResponse, error) {
	uid, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, ErrUnauthenticated
	}

	can, err := u.Enforcer.Enforce(uid, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "write")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, ErrInsufficientPermissions
	}

	if req.GetUsername() == "" {
		return nil, twirp.RequiredArgumentError("username")
	}

	if req.GetUsername() == uid {
		return nil, twirp.InvalidArgumentError("username", "cannot be the currently authenticated user")
	}

	uu, err := u.Store.GetUser(ctx, req.GetUsername())
	if err != nil {
		return nil, err
	}

	if uu == nil {
		return nil, twirp.NotFoundError("user not found")
	}

	err = u.Store.DeleteUser(ctx, uu)
	if err != nil {
		return nil, err
	}

	u.m.UsersCount().Add(-1)
	return &authv1beta1.DeleteUserResponse{
		User: &authv1beta1.User{
			Username: uu.Username,
		},
	}, nil
}
