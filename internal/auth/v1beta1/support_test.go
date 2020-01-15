// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"

	rice "github.com/GeertJohan/go.rice"
	"github.com/enseadaio/enseada/internal/auth"
	"github.com/stretchr/testify/mock"
	"go.opencensus.io/metric"
)

var model = rice.MustFindBox("../../../conf").MustString("casbin_model.conf")

const policy = `
p,test,users://user/*,read
`

type MockMetricsRegistry struct {
	mock.Mock
}

func (m *MockMetricsRegistry) UsersCount() *metric.Int64GaugeEntry {
	return new(metric.Int64GaugeEntry)
}

type MockUserStorage struct {
	mock.Mock
}

func (m *MockUserStorage) Authenticate(ctx context.Context, name string, secret string) error {
	args := m.MethodCalled("Authenticate", ctx, name, secret)
	return args.Error(0)
}

func (m *MockUserStorage) ListUsers(ctx context.Context) ([]*auth.User, error) {
	args := m.MethodCalled("ListUsers", ctx)
	return args.Get(0).([]*auth.User), args.Error(1)
}

func (m *MockUserStorage) GetUser(ctx context.Context, username string) (*auth.User, error) {
	args := m.MethodCalled("GetUser", ctx, username)
	return args.Get(0).(*auth.User), args.Error(1)
}

func (m *MockUserStorage) CreateUser(ctx context.Context, u *auth.User) error {
	args := m.MethodCalled("CreateUser", ctx, u)
	return args.Error(0)
}

func (m *MockUserStorage) UpdateUser(ctx context.Context, u *auth.User) error {
	args := m.MethodCalled("UpdateUser", ctx, u)
	return args.Error(0)
}

func (m *MockUserStorage) DeleteUser(ctx context.Context, u *auth.User) error {
	args := m.MethodCalled("DeleteUser", ctx, u)
	return args.Error(0)
}
