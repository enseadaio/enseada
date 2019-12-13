package repo

type Repo interface {
	ID() string
	StoragePath() string
}
