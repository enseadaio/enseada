// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"

	"github.com/enseadaio/enseada/internal/ctxutils"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/uber-go/tally"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/guid"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/go-kivik/kivik"
	"github.com/twitchtv/twirp"
)

type UsersAPI struct {
	logger       log.Logger
	enforcer     *casbin.Enforcer
	store        *auth.Store
	stats        tally.Scope
	usersCounter tally.Gauge
}

func NewUsersAPI(ctx context.Context, logger log.Logger, enforcer *casbin.Enforcer, store *auth.Store, stats tally.Scope) *UsersAPI {
	u := &UsersAPI{logger: logger, enforcer: enforcer, store: store, stats: stats}
	go u.updateUsersCounter(ctx)
	return u
}

func (u *UsersAPI) ListUsers(ctx context.Context, req *authv1beta1.ListUsersRequest) (*authv1beta1.ListUsersResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	can, err := u.enforcer.Enforce(id, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NewError(twirp.PermissionDenied, "")
	}

	us, err := u.store.ListUsers(ctx)
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	ups := make([]*authv1beta1.User, len(us))
	for i, u := range us {
		up := &authv1beta1.User{
			Username: u.Username,
		}
		ups[i] = up
	}

	return &authv1beta1.ListUsersResponse{
		Users: ups,
	}, nil
}

func (u *UsersAPI) GetUser(ctx context.Context, req *authv1beta1.GetUserRequest) (*authv1beta1.GetUserResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	can, err := u.enforcer.Enforce(id, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NewError(twirp.PermissionDenied, "")
	}

	if req.GetUsername() == "" {
		return nil, twirp.RequiredArgumentError("username")
	}

	user, err := u.store.GetUser(ctx, req.GetUsername())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if user == nil {
		return nil, twirp.NotFoundError("")
	}

	up := &authv1beta1.User{
		Username: user.Username,
	}

	return &authv1beta1.GetUserResponse{
		User: up,
	}, nil
}

func (u *UsersAPI) CreateUser(ctx context.Context, req *authv1beta1.CreateUserRequest) (*authv1beta1.CreateUserResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	can, err := u.enforcer.Enforce(id, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "write")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NewError(twirp.PermissionDenied, "")
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
	err = u.store.SaveUser(ctx, uu)
	if err != nil {
		// Don't like it, leaking db implementation
		if kivik.StatusCode(err) == kivik.StatusConflict {
			e := twirp.NewError(twirp.AlreadyExists, "")
			e = e.WithMeta("username", up.GetUsername())
			return nil, e
		}
		return nil, twirp.InternalErrorWith(err)
	}

	u.updateUsersCounter(ctx)
	return &authv1beta1.CreateUserResponse{
		User: up,
	}, nil
}

func (u *UsersAPI) UpdateUserPassword(ctx context.Context, req *authv1beta1.UpdateUserPasswordRequest) (*authv1beta1.UpdateUserPasswordResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	uu, err := u.store.GetUser(ctx, id)
	if err != nil {
		return nil, err
	}

	if req.GetPassword() == "" {
		return nil, twirp.RequiredArgumentError("password")
	}

	uu.Password = req.GetPassword()
	err = u.store.UpdateUser(ctx, uu)
	if err != nil {
		return nil, err
	}

	return &authv1beta1.UpdateUserPasswordResponse{}, nil
}

func (u *UsersAPI) DeleteUser(ctx context.Context, req *authv1beta1.DeleteUserRequest) (*authv1beta1.DeleteUserResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	can, err := u.enforcer.Enforce(id, guid.New(couch.UsersDB, "*", couch.KindUser).String(), "write")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NewError(twirp.PermissionDenied, "")
	}

	if req.GetUsername() == "" {
		return nil, twirp.RequiredArgumentError("username")
	}

	if req.GetUsername() == id {
		return nil, twirp.InvalidArgumentError("username", "cannot be the currently authenticated user")
	}

	uu, err := u.store.GetUser(ctx, req.GetUsername())
	if err != nil {
		return nil, err
	}

	if uu == nil {
		return nil, twirp.NotFoundError("")
	}

	err = u.store.DeleteUser(ctx, uu)
	if err != nil {
		return nil, err
	}

	u.updateUsersCounter(ctx)
	return &authv1beta1.DeleteUserResponse{
		User: &authv1beta1.User{
			Username: uu.Username,
		},
	}, nil
}

func (u *UsersAPI) updateUsersCounter(ctx context.Context) {
	if u.usersCounter == nil {
		u.usersCounter = u.stats.Gauge("users")
	}
	us, err := u.store.ListUsers(ctx)
	if err != nil {
		u.logger.Error(err)
		return
	}

	u.usersCounter.Update(float64(len(us)))
}
