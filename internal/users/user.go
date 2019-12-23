package users

type User struct {
	ID             string `json:"_id,omitempty"`
	Rev            string `json:"_rev,omitempty"`
	Username       string `json:"username"`
	Password       string `json:"-"`
	HashedPassword []byte `json:"hashed_password"`
}

func Root(pwd string) *User {
	return &User{
		Username: "root",
		Password: pwd,
	}
}
