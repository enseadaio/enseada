// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package enseada

import (
	"net/http"

	authv1beta1 "github.com/enseadaio/enseada/rpc/auth/v1beta1"
	mavenv1beta1 "github.com/enseadaio/enseada/rpc/maven/v1beta1"
	"github.com/twitchtv/twirp"
)

type Client interface {
	AclV1Beta1() authv1beta1.AclAPI
	UsersV1Beta1() authv1beta1.UsersAPI
	OAuthClientsV1Beta1() authv1beta1.OAuthClientsAPI
	MavenV1Beta1() mavenv1beta1.MavenAPI
}

type client struct {
	url    string
	hc     *http.Client
	twOpts []twirp.ClientOption
}

func NewClient(url string, hc *http.Client, opts ...twirp.ClientOption) Client {
	return &client{
		url:    url,
		hc:     hc,
		twOpts: opts,
	}
}

func (c *client) AclV1Beta1() authv1beta1.AclAPI {
	return authv1beta1.NewAclAPIProtobufClient(c.url, c.hc, c.twOpts...)
}

func (c *client) UsersV1Beta1() authv1beta1.UsersAPI {
	return authv1beta1.NewUsersAPIProtobufClient(c.url, c.hc, c.twOpts...)
}

func (c *client) OAuthClientsV1Beta1() authv1beta1.OAuthClientsAPI {
	return authv1beta1.NewOAuthClientsAPIProtobufClient(c.url, c.hc, c.twOpts...)
}

func (c *client) MavenV1Beta1() mavenv1beta1.MavenAPI {
	return mavenv1beta1.NewMavenAPIProtobufClient(c.url, c.hc, c.twOpts...)
}
