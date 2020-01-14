// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"
	"testing"

	"github.com/casbin/casbin/v2"
	camodel "github.com/casbin/casbin/v2/model"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/enseadaio/enseada/internal/ctxutils"
	"github.com/enseadaio/enseada/pkg/log"
	"github.com/enseadaio/enseada/pkg/log/adapters"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	scas "github.com/qiangmzsx/string-adapter/v2"
	"github.com/stretchr/testify/suite"
)

type UsersAPISuite struct {
	suite.Suite
	api authv1beta1.UsersAPI
	us  *MockUserStorage
	mr  *MockMetricsRegistry
}

func (s *UsersAPISuite) SetupSuite() {
	l, err := adapters.NewZapLoggerAdapter(log.DEBUG)
	s.Require().NoError(err)

	m, err := camodel.NewModelFromString(model)
	s.Require().NoError(err)

	enf, err := casbin.NewEnforcer(m, scas.NewAdapter(policy))
	s.Require().NoError(err)

	s.us = new(MockUserStorage)
	s.mr = new(MockMetricsRegistry)
	s.api = NewUsersAPI(l, enf, s.us, s.mr)
}

func (s *UsersAPISuite) TestListUsers() {
	ctx := ctxutils.WithCurrentUserID(context.TODO(), "test")

	s.us.On("ListUsers", ctx).Return([]*auth.User{
		{Username: "test"},
	}, nil).Once()
	res, err := s.api.ListUsers(ctx, new(authv1beta1.ListUsersRequest))
	s.Require().NoError(err)

	s.us.AssertExpectations(s.T())

	users := res.GetUsers()
	s.Require().NotNil(users)

	s.Require().Equal(1, len(users))
	u := users[0]
	s.Require().Equal(u.Username, "test")
}

func (s *UsersAPISuite) TestListUsersEmpty() {
	ctx := ctxutils.WithCurrentUserID(context.TODO(), "test")

	s.us.On("ListUsers", ctx).Return([]*auth.User{}, nil).Once()
	res, err := s.api.ListUsers(ctx, new(authv1beta1.ListUsersRequest))
	s.Require().Nil(res)
	s.Require().Error(err)
	s.Require().Equal(ErrNoUsersFound, err)
	s.us.AssertExpectations(s.T())
}

func (s *UsersAPISuite) TestListUsersUnauthenticated() {
	ctx := context.TODO()

	res, err := s.api.ListUsers(ctx, new(authv1beta1.ListUsersRequest))
	s.Require().Nil(res)
	s.Require().Error(err)
	s.Require().Equal(ErrUnauthenticated, err)
	s.us.AssertExpectations(s.T())
}

func (s *UsersAPISuite) TestListUsersInsufficientPermissions() {
	ctx := ctxutils.WithCurrentUserID(context.TODO(), "another")

	res, err := s.api.ListUsers(ctx, new(authv1beta1.ListUsersRequest))
	s.Require().Nil(res)
	s.Require().Error(err)
	s.Require().Equal(ErrInsufficientPermissions, err)
	s.us.AssertExpectations(s.T())
}

func TestUsersAPI(t *testing.T) {
	suite.Run(t, new(UsersAPISuite))
}
