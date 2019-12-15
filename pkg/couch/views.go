package couch

import (
	"context"
	"github.com/go-kivik/kivik"
	"github.com/labstack/gommon/log"
	"net/http"
)

func initViews(ctx context.Context, client *kivik.Client) error {
	if err := initFilesIndexView(ctx, client); err != nil {
		return err
	}
	return nil
}

func initFilesIndexView(ctx context.Context, client *kivik.Client) error {
	db := client.DB(ctx, "maven2")
	row := db.Get(ctx, "_design/files")
	if row.Err != nil && kivik.StatusCode(row.Err) != http.StatusNotFound {
		return row.Err
	}

	doc := map[string]interface{}{
		"_id": "_design/files",
		"views": map[string]interface{}{
			"repo_files_index": map[string]interface{}{
				"map": `
function(doc) {
  if (doc.kind === "repository" && doc.files && Array.isArray(doc.files)) {
    for (var i in doc.files) {
      emit(doc.files[i], null)
    }  
  }
}
`,
			},
		},
	}

	if row.Rev != "" {
		doc["_rev"] = row.Rev
	}

	_, err := db.Put(ctx, "_design/files", doc)

	if kivik.StatusCode(err) == kivik.StatusConflict {
		log.Info("repo_files_index view already exists")
		return nil
	} else {
		log.Info("initializing repo_files_index view")
		return err
	}
}
