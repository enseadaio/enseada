package mavensvcv1beta1

import (
	"context"
	"fmt"
	"github.com/enseadaio/enseada/internal/maven"
	"github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/enseadaio/enseada/rpc/meta/v1beta1"
	"github.com/twitchtv/twirp"
)

var RepoNotFoundError = func(id string) twirp.Error {
	return twirp.NotFoundError(fmt.Sprintf("no Maven repository found by id %s", id))
}

type Service struct {
	Maven *maven.Maven
}

func (s Service) ListRepos(ctx context.Context, req *mavenv1beta1.ListReposRequest) (*mavenv1beta1.ListReposResponse, error) {
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

func (s Service) GetRepo(ctx context.Context, req *mavenv1beta1.GetRepoRequest) (*mavenv1beta1.GetRepoResponse, error) {
	id := req.GetId()
	if id == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	repo, err := s.Maven.GetRepo(ctx, id)
	if err != nil {
		return nil, err
	}

	if repo == nil {
		return nil, RepoNotFoundError(id)
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

func (s Service) CreateRepo(ctx context.Context, req *mavenv1beta1.CreateRepoRequest) (*mavenv1beta1.CreateRepoResponse, error) {
	if req.GetGroupId() == "" {
		return nil, twirp.RequiredArgumentError("group_id")
	}

	if req.GetArtifactId() == "" {
		return nil, twirp.RequiredArgumentError("artifact_id")
	}

	repo := maven.NewRepo(req.GroupId, req.ArtifactId)
	err := s.Maven.InitRepo(ctx, &repo)
	if err != nil {
		if err == maven.ErrorRepoAlreadyPresent {
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

func (s Service) DeleteRepo(ctx context.Context, req *mavenv1beta1.DeleteRepoRequest) (*mavenv1beta1.DeleteRepoResponse, error) {
	id := req.GetId()
	if id == "" {
		return nil, twirp.RequiredArgumentError("id")
	}

	repo, err := s.Maven.DeleteRepo(ctx, id)
	if err != nil {
		return nil, err
	}

	if repo == nil {
		return nil, RepoNotFoundError(id)
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
