package oauth

import (
	"github.com/enseadaio/enseada/internal/couch"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
	"net/url"
	"time"
)

type RequestWrapper struct {
	ID      string     `json:"_id,omitempty"`
	Rev     string     `json:"_rev,omitempty"`
	Kind    couch.Kind `json:"kind"`
	Req     *Request   `json:"req"`
	Revoked bool       `json:"revoked,omitempty"`
	Sig     string     `json:"sig,omitempty"`
}

type Request struct {
	ID                string                 `json:"id"`
	RequestedAt       time.Time              `json:"requested_at"`
	Client            *Client                `json:"client"`
	RequestedScopes   fosite.Arguments       `json:"scopes"`
	GrantedScopes     fosite.Arguments       `json:"granted_scopes"`
	Form              url.Values             `json:"form"`
	Session           *openid.DefaultSession `json:"session"`
	RequestedAudience fosite.Arguments       `json:"requested_audience"`
	GrantedAudience   fosite.Arguments       `json:"granted_audience"`
}

func (r *Request) SetID(id string) {
	r.ID = id
}

func (r *Request) GetID() string {
	return r.ID
}

func (r *Request) GetRequestedAt() time.Time {
	return r.RequestedAt
}

func (r *Request) GetClient() fosite.Client {
	return r.Client
}

func (r *Request) GetRequestedScopes() fosite.Arguments {
	return r.RequestedScopes
}

func (r *Request) GetRequestedAudience() fosite.Arguments {
	return r.RequestedAudience
}

func (r *Request) SetRequestedScopes(scopes fosite.Arguments) {
	r.RequestedScopes = scopes
}

func (r *Request) SetRequestedAudience(audience fosite.Arguments) {
	r.RequestedAudience = audience
}

func (r *Request) AppendRequestedScope(scope string) {
	for _, has := range r.RequestedScopes {
		if scope == has {
			return
		}
	}
	r.RequestedScopes = append(r.RequestedScopes, scope)
}

func (r *Request) GetGrantedScopes() fosite.Arguments {
	return r.GrantedScopes
}

func (r *Request) GetGrantedAudience() fosite.Arguments {
	return r.GrantedAudience
}

func (r *Request) GrantScope(scope string) {
	for _, has := range r.GrantedScopes {
		if scope == has {
			return
		}
	}
	r.GrantedScopes = append(r.GrantedScopes, scope)
}

func (r *Request) GrantAudience(audience string) {
	for _, has := range r.GrantedAudience {
		if audience == has {
			return
		}
	}
	r.GrantedAudience = append(r.GrantedAudience, audience)
}

func (r *Request) GetSession() fosite.Session {
	return r.Session
}

func (r *Request) SetSession(session fosite.Session) {
	r.Session = session.(*openid.DefaultSession)
}

func (r *Request) GetRequestForm() url.Values {
	return r.Form
}

func (r *Request) Merge(request fosite.Requester) {
	r.RequestedScopes = request.GetRequestedScopes()
	r.GrantedScopes = request.GetGrantedScopes()
	r.RequestedAudience = request.GetRequestedAudience()
	r.GrantedAudience = request.GetGrantedAudience()
	r.RequestedAt = request.GetRequestedAt()
	r.Client = request.GetClient().(*Client)
	r.SetSession(request.GetSession())
	r.Form = request.GetRequestForm()
}

func (r *Request) Sanitize(allowedParameters []string) fosite.Requester {
	return r
}
