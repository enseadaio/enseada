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

func (r *Service) ListRepos(ctx context.Context) ([]HTTPRepo, error) {
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

	repos := make([]HTTPRepo, 0)
	for rows.Next() {
		repo := make(HTTPRepo)
		if err := rows.ScanDoc(&repo); err != nil {
			return nil, err
		}

		repo = ToHTTPJson(repo)
		repos = append(repos, repo)
	}
	if rows.Err() != nil {
		return nil, rows.Err()
	}

	r.Logger.Infof("Found %d repositories", len(repos))
	return repos, nil
}

func (r *Service) GetRepo(ctx context.Context, id string) (HTTPRepo, error) {
	db := r.Data.DB(ctx, "repositories")
	var repo HTTPRepo
	row := db.Get(ctx, id)
	if err := row.ScanDoc(&repo); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}
		return nil, err
	}

	repo = ToHTTPJson(repo)

	return repo, nil
}
