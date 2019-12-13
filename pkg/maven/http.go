package maven

type HTTPRepo struct {
	Type       string `json:"type"`
	GroupID    string `json:"group_id"`
	ArtifactID string `json:"artifact_id"`
}

func RepoToHTTPJson(repo *Repo) HTTPRepo {
	return HTTPRepo{
		Type:       repo.Type,
		GroupID:    repo.GroupID,
		ArtifactID: repo.ArtifactID,
	}
}
