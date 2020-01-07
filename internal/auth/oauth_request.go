// Copyright 2019 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"net/url"
	"time"

	"github.com/enseadaio/enseada/internal/couch"
	"github.com/ory/fosite"
	"github.com/ory/fosite/handler/openid"
)

type OAuthRequestWrapper struct {
	ID      string        `json:"_id,omitempty"`
	Rev     string        `json:"_rev,omitempty"`
	Kind    couch.Kind    `json:"kind"`
	Req     *OAuthRequest `json:"req"`
	Revoked bool          `json:"revoked,omitempty"`
	Sig     string        `json:"sig,omitempty"`
}

type OAuthRequest struct {
	ID                string                 `json:"id"`
	RequestedAt       time.Time              `json:"requested_at"`
	Client            *OAuthClient           `json:"client"`
	RequestedScopes   fosite.Arguments       `json:"scopes"`
	GrantedScopes     fosite.Arguments       `json:"granted_scopes"`
	Form              url.Values             `json:"form"`
	Session           *openid.DefaultSession `json:"session"`
	RequestedAudience fosite.Arguments       `json:"requested_audience"`
	GrantedAudience   fosite.Arguments       `json:"granted_audience"`
}

func (r *OAuthRequest) SetID(id string) {
	r.ID = id
}

func (r *OAuthRequest) GetID() string {
	return r.ID
}

func (r *OAuthRequest) GetRequestedAt() time.Time {
	return r.RequestedAt
}

func (r *OAuthRequest) GetClient() fosite.Client {
	return r.Client
}

func (r *OAuthRequest) GetRequestedScopes() fosite.Arguments {
	return r.RequestedScopes
}

func (r *OAuthRequest) GetRequestedAudience() fosite.Arguments {
	return r.RequestedAudience
}

func (r *OAuthRequest) SetRequestedScopes(scopes fosite.Arguments) {
	r.RequestedScopes = scopes
}

func (r *OAuthRequest) SetRequestedAudience(audience fosite.Arguments) {
	r.RequestedAudience = audience
}

func (r *OAuthRequest) AppendRequestedScope(scope string) {
	for _, has := range r.RequestedScopes {
		if scope == has {
			return
		}
	}
	r.RequestedScopes = append(r.RequestedScopes, scope)
}

func (r *OAuthRequest) GetGrantedScopes() fosite.Arguments {
	return r.GrantedScopes
}

func (r *OAuthRequest) GetGrantedAudience() fosite.Arguments {
	return r.GrantedAudience
}

func (r *OAuthRequest) GrantScope(scope string) {
	for _, has := range r.GrantedScopes {
		if scope == has {
			return
		}
	}
	r.GrantedScopes = append(r.GrantedScopes, scope)
}

func (r *OAuthRequest) GrantAudience(audience string) {
	for _, has := range r.GrantedAudience {
		if audience == has {
			return
		}
	}
	r.GrantedAudience = append(r.GrantedAudience, audience)
}

func (r *OAuthRequest) GetSession() fosite.Session {
	return r.Session
}

func (r *OAuthRequest) SetSession(session fosite.Session) {
	r.Session = session.(*openid.DefaultSession)
}

func (r *OAuthRequest) GetRequestForm() url.Values {
	return r.Form
}

func (r *OAuthRequest) Merge(request fosite.Requester) {
	r.RequestedScopes = request.GetRequestedScopes()
	r.GrantedScopes = request.GetGrantedScopes()
	r.RequestedAudience = request.GetRequestedAudience()
	r.GrantedAudience = request.GetGrantedAudience()
	r.RequestedAt = request.GetRequestedAt()
	r.Client = request.GetClient().(*OAuthClient)
	r.SetSession(request.GetSession())
	r.Form = request.GetRequestForm()
	r.ID = request.GetID()
}

func (r *OAuthRequest) Sanitize(allowedParameters []string) fosite.Requester {
	return r
}
