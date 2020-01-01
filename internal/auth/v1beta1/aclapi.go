// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package authv1beta1api

import (
	"context"
	"github.com/enseadaio/enseada/internal/middleware"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/guid"
	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	"github.com/labstack/echo"
	"github.com/twitchtv/twirp"
)

type ACLService struct {
	Logger   echo.Logger
	Enforcer *casbin.Enforcer
}

func (s ACLService) ListRules(ctx context.Context, req *authv1beta1.ListRulesRequest) (*authv1beta1.ListRulesResponse, error) {
	g, ok := middleware.CurrentUserGUID(ctx)
	if !ok {
		s.Logger.Info("no current user")
	} else {
		s.Logger.Infof("current user: %s", g.String())
	}
	policy := s.Enforcer.GetPolicy()
	var rules []*authv1beta1.AclRule

	for _, r := range policy {
		var rule authv1beta1.AclRule
		if len(r) > 0 {
			rule.Sub = r[0]
		}
		if len(r) > 1 {
			rule.Obj = r[1]
		}
		if len(r) > 2 {
			rule.Act = r[2]
		}
		rules = append(rules, &rule)
	}
	return &authv1beta1.ListRulesResponse{
		Rules: rules,
	}, nil
}

func (s ACLService) AddRule(ctx context.Context, req *authv1beta1.AddRuleRequest) (*authv1beta1.AddRuleResponse, error) {
	rule := req.Rule
	if rule == nil {
		return nil, twirp.RequiredArgumentError("rule")
	}

	if _, err := guid.Parse(rule.Sub); err != nil {
		return nil, twirp.InvalidArgumentError("sub", err.Error())
	}

	if _, err := guid.Parse(rule.Obj); err != nil {
		return nil, twirp.InvalidArgumentError("sub", err.Error())
	}

	if rule.Act == "" {
		return nil, twirp.RequiredArgumentError("act")
	}

	ok, err := s.Enforcer.AddPolicy(rule.Sub, rule.Obj, rule.Act)
	if err != nil {
		return nil, err
	}

	if ok {
		return &authv1beta1.AddRuleResponse{Rule: rule}, nil
	}

	return nil, twirp.NewError(twirp.AlreadyExists, "")
}
