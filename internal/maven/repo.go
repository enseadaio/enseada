package maven

import (
	"bytes"
	"context"
	"crypto/md5"
	"crypto/sha1"
	"fmt"
	"github.com/enseadaio/enseada/pkg/couch"
	"github.com/go-kivik/kivik"
	"net/http"
	"strings"
	"text/template"
	"time"
)

const baseMetadataFile = `
<?xml version="1.0" encoding="UTF-8"?>
<metadata>
	<groupId>{{ .GroupId }}</groupId>
	<artifactId>{{ .ArtifactId }}</artifactId>
	<versioning>
		<versions></versions>
		<lastUpdated>{{ .TimeStamp }}</lastUpdated>
	</versioning>
</metadata>
`

type Repo struct {
	Id         string     `json:"_id,omitempty"`
	Rev        string     `json:"_rev,omitempty"`
	GroupID    string     `json:"group_id"`
	ArtifactID string     `json:"artifact_id"`
	StorePath  string     `json:"storage_path"`
	Files      []string   `json:"files"`
	Kind       couch.Kind `json:"kind"`
}

func (r *Repo) ID() string {
	return r.Id
}

func (r *Repo) StoragePath() string {
	return r.StorePath
}

func NewRepo(groupID string, artifactID string) Repo {
	group := strings.ReplaceAll(groupID, ".", "/")
	return Repo{
		Id:         repoID(groupID, artifactID),
		GroupID:    groupID,
		ArtifactID: artifactID,
		StorePath:  strings.Join([]string{group, artifactID}, "/"),
		Kind:       couch.KindRepository,
	}
}

func (m *Maven) ListRepos(ctx context.Context) ([]*Repo, error) {
	db := m.Data.DB(ctx, "maven2")
	rows, err := db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"kind": "repository",
		},
	})

	if err != nil {
		return nil, err
	}

	repos := make([]*Repo, 0)
	for rows.Next() {
		var repo Repo
		if err := rows.ScanDoc(&repo); err != nil {
			return nil, err
		}
		repos = append(repos, &repo)
	}
	if rows.Err() != nil {
		return nil, err
	}

	return repos, nil
}

func (m *Maven) GetRepo(ctx context.Context, id string) (*Repo, error) {
	db := m.Data.DB(ctx, "maven2")
	row := db.Get(ctx, id)
	repo := &Repo{}
	if err := row.ScanDoc(repo); err != nil {
		if kivik.StatusCode(err) == kivik.StatusNotFound {
			return nil, nil
		}
		return nil, err
	}
	return repo, nil
}

func (m *Maven) FindRepo(ctx context.Context, groupID string, artifactID string) (*Repo, error) {
	return m.GetRepo(ctx, repoID(groupID, artifactID))
}

func (m *Maven) SaveRepo(ctx context.Context, repo *Repo) error {
	db := m.Data.DB(ctx, "maven2")
	rev, err := db.Put(ctx, repo.Id, repo)
	if err != nil {
		return err
	}
	repo.Rev = rev
	return err
}

func (m *Maven) DeleteRepo(ctx context.Context, id string) (*Repo, error) {
	db := m.Data.DB(ctx, "maven2")
	repo, err := m.GetRepo(ctx, id)
	if err != nil || repo == nil {
		return nil, err
	}

	rev, err := db.Delete(ctx, repo.Id, repo.Rev)
	if err != nil {
		return nil, err
	}

	repo.Rev = rev
	return repo, nil
}

func repoID(groupID string, artifactID string) string {
	return strings.Join([]string{groupID, artifactID}, ":")
}

func fromId(id string) (Repo, error) {
	parts := strings.Split(id, ":")
	if len(parts) != 2 {
		return Repo{}, ErrorInvalidRepoId(id)
	}
	return NewRepo(parts[0], parts[1]), nil
}

func (m *Maven) InitRepo(ctx context.Context, repo *Repo) error {
	db := m.Data.DB(ctx, "maven2")

	m.Logger.Infof("Initializing repo %s", repo.ID)
	err := save(ctx, db, repo)
	if err != nil {
		return err
	}

	m.Logger.Infof("Created repo %s", repo.ID)
	t, err := template.New("maven-metadata.xml").Parse(baseMetadataFile)
	if err != nil {
		return err
	}

	var buf bytes.Buffer
	err = t.Execute(&buf, map[string]interface{}{
		"GroupId":    repo.GroupID,
		"ArtifactId": repo.ArtifactID,
		"TimeStamp":  time.Now().Unix(),
	})
	if err != nil {
		return err
	}

	m.Logger.Infof("Creating file %s", t.ParseName)
	file := &RepoFile{
		Repo:     repo,
		Filename: t.ParseName,
		Content:  buf.Bytes(),
	}

	md5sum := &RepoFile{
		Repo:     repo,
		Filename: fmt.Sprintf("%s.md5", t.ParseName),
		Content:  []byte(fmt.Sprintf("%x", md5.Sum(file.Content))),
	}

	sha1sum := &RepoFile{
		Repo:     repo,
		Filename: fmt.Sprintf("%s.sha1", t.ParseName),
		Content:  []byte(fmt.Sprintf("%x", sha1.Sum(file.Content))),
	}

	path := filePath(file)
	repo.Files = append(repo.Files, path)
	err = m.PutFile(ctx, path, file.Content)
	if err != nil {
		return err
	}

	path = filePath(md5sum)
	repo.Files = append(repo.Files, path)
	err = m.PutFile(ctx, path, md5sum.Content)
	if err != nil {
		return err
	}

	path = filePath(sha1sum)
	repo.Files = append(repo.Files, path)
	err = m.PutFile(ctx, path, sha1sum.Content)
	if err != nil {
		return err
	}

	return save(ctx, db, repo)
}

func save(ctx context.Context, db *kivik.DB, repo *Repo) error {
	rev, err := db.Put(ctx, repo.Id, repo)
	if err != nil {
		switch kivik.StatusCode(err) {
		case http.StatusConflict:
			return ErrorRepoAlreadyPresent
		default:
			return err
		}
	}
	repo.Rev = rev
	return nil
}
