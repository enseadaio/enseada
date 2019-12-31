// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"context"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	metav1beta1 "github.com/enseadaio/enseada/rpc/meta/v1beta1"
	"github.com/twitchtv/twirp"
)

type ServiceV1Beta1 struct {
	Maven *Maven
}

func (s ServiceV1Beta1) ListRepos(ctx context.Context, req *mavenv1beta1.ListReposRequest) (*mavenv1beta1.ListReposResponse, error) {
	repos, err := s.Maven.ListRepos(ctx)
	if err != nil {
		return nil, err
	}

	rr := make([]*mavenv1beta1.Repo, len(repos))
	for i, repo := range repos {
		r := &mavenv1beta1.Repo{
			Metadata: &metav1beta1.Metadata{
				Name: repo.Id,
			},
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		}
		rr[i] = r
	}

	return &mavenv1beta1.ListReposResponse{
		Repos: rr,
	}, nil
}

func (s ServiceV1Beta1) GetRepo(ctx context.Context, req *mavenv1beta1.GetRepoRequest) (*mavenv1beta1.GetRepoResponse, error) {
	id := req.GetId()
	if id == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	repo, err := s.Maven.GetRepo(ctx, id)
	if err != nil {
		return nil, err
	}

	if repo == nil {
		return nil, TwirpRepoNotFoundError(id)
	}

	return &mavenv1beta1.GetRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Metadata: &metav1beta1.Metadata{
				Name: repo.Id,
			},
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}

func (s ServiceV1Beta1) CreateRepo(ctx context.Context, req *mavenv1beta1.CreateRepoRequest) (*mavenv1beta1.CreateRepoResponse, error) {
	if req.GetGroupId() == "" {
		return nil, twirp.RequiredArgumentError("group_id")
	}

	if req.GetArtifactId() == "" {
		return nil, twirp.RequiredArgumentError("artifact_id")
	}

	repo := NewRepo(req.GroupId, req.ArtifactId)
	err := s.Maven.InitRepo(ctx, &repo)
	if err != nil {
		if err == ErrorRepoAlreadyPresent {
			return nil, twirp.NewError(twirp.AlreadyExists, "Maven repository already present")
		}
		return nil, err
	}

	return &mavenv1beta1.CreateRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Metadata: &metav1beta1.Metadata{
				Name: repo.Id,
			},
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}

func (s ServiceV1Beta1) DeleteRepo(ctx context.Context, req *mavenv1beta1.DeleteRepoRequest) (*mavenv1beta1.DeleteRepoResponse, error) {
	id := req.GetId()
	if id == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	repo, err := s.Maven.DeleteRepo(ctx, id)
	if err != nil {
		return nil, err
	}

	if repo == nil {
		return nil, TwirpRepoNotFoundError(id)
	}

	return &mavenv1beta1.DeleteRepoResponse{
		Repo: &mavenv1beta1.Repo{
			Metadata: &metav1beta1.Metadata{
				Name: id,
			},
			GroupId:    repo.GroupID,
			ArtifactId: repo.ArtifactID,
		},
	}, nil
}
