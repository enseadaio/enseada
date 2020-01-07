// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package mavenv1beta1api

import (
	"context"

	"github.com/enseadaio/enseada/internal/ctxutils"

	"github.com/casbin/casbin/v2"
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/enseadaio/enseada/internal/guid"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/enseadaio/enseada/internal/scope"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/twitchtv/twirp"
)

type Service struct {
	Maven    *maven.Maven
	Enforcer *casbin.Enforcer
}

func (s Service) ListRepos(ctx context.Context, req *mavenv1beta1.ListReposRequest) (*mavenv1beta1.ListReposResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !scopes.Has(scope.MavenRepoRead) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	var repos []*maven.Repo
	if id == "root" {
		rs, err := s.Maven.ListRepos(ctx, couch.Query{})
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}

		repos = rs
	} else {
		ps := s.Enforcer.GetPermissionsForUser(id)
		ids := make([]string, 0)
		for _, p := range ps {
			g, err := guid.Parse(p[1])
			if err != nil {
				return nil, twirp.InternalErrorWith(err)
			}

			if g.DB() == couch.MavenDB && g.Kind() == couch.KindRepository && p[2] == "read" {
				ids = append(ids, g.ID())
			}
		}

		rs, err := s.Maven.ListRepos(ctx, couch.Query{
			"_id": couch.Query{
				"$in": ids,
			},
		})
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}

		repos = rs
	}
	rs := make([]*mavenv1beta1.Repo, len(repos))
	for i, repo := range repos {
		r := &mavenv1beta1.Repo{
			Id:         repo.ID,
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		}
		rs[i] = r
	}

	return &mavenv1beta1.ListReposResponse{
		Repos: rs,
	}, nil
}

func (s Service) GetRepo(ctx context.Context, req *mavenv1beta1.GetRepoRequest) (*mavenv1beta1.GetRepoResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !scopes.Has(scope.MavenRepoRead) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.MavenDB, req.GetId(), couch.KindRepository)
	can, err := s.Enforcer.Enforce(id, cg.String(), "read")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	repo, err := s.Maven.GetRepo(ctx, req.GetId())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if repo == nil {
		return nil, twirp.NotFoundError("")
	}

	return &mavenv1beta1.GetRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Id:         repo.ID,
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}

func (s Service) CreateRepo(ctx context.Context, req *mavenv1beta1.CreateRepoRequest) (*mavenv1beta1.CreateRepoResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !scopes.Has(scope.MavenRepoWrite) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetGroupId() == "" {
		return nil, twirp.RequiredArgumentError("group_id")
	}

	if req.GetArtifactId() == "" {
		return nil, twirp.RequiredArgumentError("artifact_id")
	}

	repo := maven.NewRepo(req.GroupId, req.ArtifactId)
	err := s.Maven.InitRepo(ctx, &repo)
	if err != nil {
		if err == maven.ErrRepoAlreadyPresent {
			return nil, twirp.NewError(twirp.AlreadyExists, "Maven repository already present")
		}
		return nil, twirp.InternalErrorWith(err)
	}

	cg := guid.New(couch.MavenDB, repo.ID, couch.KindRepository)
	ps := []string{"read", "update", "write", "delete"}
	for _, p := range ps {
		_, err := s.Enforcer.AddPermissionForUser(id, cg.String(), p)
		if err != nil {
			return nil, twirp.InternalErrorWith(err)
		}
	}

	return &mavenv1beta1.CreateRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Id:         repo.ID,
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}

func (s Service) DeleteRepo(ctx context.Context, req *mavenv1beta1.DeleteRepoRequest) (*mavenv1beta1.DeleteRepoResponse, error) {
	id, ok := ctxutils.CurrentUserID(ctx)
	if !ok {
		return nil, twirp.NewError(twirp.Unauthenticated, "")
	}

	scopes, _ := ctxutils.Scopes(ctx)
	if !scopes.Has(scope.MavenRepoWrite) {
		return nil, twirp.NewError(twirp.PermissionDenied, "insufficient scopes")
	}

	if req.GetId() == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	cg := guid.New(couch.MavenDB, req.GetId(), couch.KindRepository)
	can, err := s.Enforcer.Enforce(id, cg.String(), "delete")
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if !can {
		return nil, twirp.NotFoundError("")
	}

	repo, err := s.Maven.DeleteRepo(ctx, req.GetId())
	if err != nil {
		return nil, twirp.InternalErrorWith(err)
	}

	if repo == nil {
		return nil, twirp.NotFoundError("")
	}

	return &mavenv1beta1.DeleteRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Id:         repo.ID,
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}
