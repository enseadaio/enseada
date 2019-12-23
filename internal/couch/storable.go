package couch

type Storable interface {
	GetID() string
	GetRev() string
	SetRev(rev string)
}
