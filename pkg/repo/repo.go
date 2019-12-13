package repo

type HTTPRepo map[string]interface{}

func ToHTTPJson(repo HTTPRepo) HTTPRepo {
	repo["id"] = repo["_id"]
	delete(repo, "_id")
	delete(repo, "_rev")
	delete(repo, "storage_path")
	return repo
}
