package repo

import (
	"context"
	"github.com/go-kivik/kivik"
	"github.com/labstack/echo"
)

type Service struct {
	Logger echo.Logger
	Data   *kivik.Client
}

type R map[string]interface{}

func (r *Service) ListRepos(ctx context.Context) ([]R, error) {
	r.Logger.Infof("Listing repositories")
	db := r.Data.DB(ctx, "repositories")
	rows, err := db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"kind": "repository",
		},
	})

	if err != nil {
		return nil, err
	}

	repos := make([]R, 0)
	for rows.Next() {
		repo := make(R)
		if err := rows.ScanDoc(&repo); err != nil {
			return nil, err
		}

		repo["id"] = repo["_id"]
		delete(repo, "_id")
		delete(repo, "_rev")
		delete(repo, "storage_path")

		repos = append(repos, repo)
		r.Logger.Warnf("%+v", repo)
	}
	if rows.Err() != nil {
		return nil, rows.Err()
	}

	r.Logger.Infof("Found %d repositories", len(repos))
	return repos, nil
}

func (r *Service) GetRepo(ctx context.Context, id string) (R, error) {
	db := r.Data.DB(ctx, "repositories")
	var repo R
	row := db.Get(ctx, id)
	if err := row.ScanDoc(&repo); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}
		return nil, err
	}

	return repo, nil
}
