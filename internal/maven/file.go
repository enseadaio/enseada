// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package maven

import (
	"context"
	"fmt"
	"sort"
	"strings"
)

const StoragePrefix = "maven2"

type RepoFile struct {
	Repo     *Repo
	Filename string
	Version  string
	Content  []byte
}

func (m *Maven) GetFile(ctx context.Context, path string) (*RepoFile, error) {
	m.logger.Infof("looking up file with path %s", fmt.Sprintf(`"%s"`, path))
	db := m.client.DB(ctx, "maven2")
	rows, err := db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"files": map[string]interface{}{
				"$elemMatch": map[string]interface{}{
					"$eq": path,
				},
			},
		},
	})

	if err != nil {
		return nil, err
	}

	m.logger.Infof("found %d files with path %s", rows.TotalRows(), path)

	var repoId string
	fileCount := 0
	for rows.Next() {
		if repoId != "" {
			continue
		}

		d := make(map[string]interface{})
		if err := rows.ScanDoc(&d); err != nil {
			return nil, err
		}
		repoId = d["_id"].(string)
		m.logger.Infof("found matching repo %s", repoId)
		fileCount++
	}
	if fileCount == 0 {
		m.logger.Warnf("no file found with path %s", path)
		return nil, nil
	}

	if fileCount > 1 {
		m.logger.Warnf("too many files found with path %s, actual %d", path, fileCount)
		return nil, ErrorTooManyFilesForKey(1, fileCount)
	}

	obj, err := m.storage.GetObject(path)
	if err != nil {
		return nil, err
	}

	repo, err := fromId(repoId)
	if err != nil {
		return nil, err
	}

	slices := strings.Split(path, "/")
	return &RepoFile{
		Repo:     &repo,
		Filename: slices[len(slices)-1],
		Content:  obj.Content,
	}, nil
}

func (m *Maven) PutFile(ctx context.Context, path string, content []byte) error {
	return m.storage.PutObject(path, content)
}

func (m *Maven) PutRepoFile(ctx context.Context, path string, content []byte) (*RepoFile, error) {
	db := m.client.DB(ctx, "maven2")
	rows, err := db.Find(ctx, map[string]interface{}{
		"selector": map[string]interface{}{
			"kind": "repository",
			"type": "maven",
		},
	})
	if err != nil {
		return nil, err
	}

	var repo Repo
	for rows.Next() {
		var r Repo
		if err := rows.ScanDoc(&r); err != nil {
			return nil, err
		}
		if strings.HasPrefix(path, r.StoragePath()) {
			repo = r
			break
		}
	}

	if repo.Id == "" {
		return nil, ErrorRepoNotFound
	}

	trimmed := strings.TrimPrefix(path, repo.StoragePath())
	trimmed = strings.TrimPrefix(trimmed, "/")
	slices := strings.Split(trimmed, "/")
	filename := slices[len(slices)-1]
	var version string
	if len(slices) == 2 {
		version = slices[0]
	}
	file := &RepoFile{
		Repo:     &repo,
		Filename: filename,
		Version:  version,
		Content:  content,
	}
	m.logger.Infof("storing file %+v", file)
	spath := filePath(file)
	err = m.PutFile(ctx, spath, file.Content)
	if err != nil {
		return nil, err
	}

	repo.Files = append(repo.Files, spath)
	in := repo.Files
	sort.Strings(in)
	j := 0
	for i := 1; i < len(in); i++ {
		if in[j] == in[i] {
			continue
		}
		j++
		in[j] = in[i]
	}
	repo.Files = in[:j+1]
	return file, m.SaveRepo(ctx, &repo)
}

func filePath(file *RepoFile) string {
	repo := file.Repo
	if file.Version == "" {
		return fmt.Sprintf("%s/%s/%s", StoragePrefix, repo.StoragePath(), file.Filename)
	} else {
		return fmt.Sprintf("%s/%s/%s/%s", StoragePrefix, repo.StoragePath(), file.Version, file.Filename)
	}
}
