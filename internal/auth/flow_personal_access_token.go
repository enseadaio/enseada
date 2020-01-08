// Copyright 2019-2020 Enseada authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package auth

import (
	"context"
	"time"

	"github.com/ory/fosite"
	"github.com/ory/fosite/compose"
	"github.com/ory/fosite/handler/oauth2"
	"github.com/pkg/errors"
)

func PersonalAccessTokenFactory(config *compose.Config, storage interface{}, strategy interface{}) interface{} {
	return &PersonalAccessTokenHandler{
		HandleHelper: &oauth2.HandleHelper{
			AccessTokenStrategy:  strategy.(oauth2.AccessTokenStrategy),
			AccessTokenStorage:   storage.(oauth2.AccessTokenStorage),
			AccessTokenLifespan:  3153600000 * time.Second, // 100 years
			RefreshTokenLifespan: -1,
		},
		ScopeStrategy:            config.GetScopeStrategy(),
		AudienceMatchingStrategy: config.GetAudienceStrategy(),
	}
}

// PersonalAccessTokenHandler is a fosite.Handler that implements
// the custom personal_access_token grant type for long-lived authorization tokens used by machine accounts
type PersonalAccessTokenHandler struct {
	*oauth2.HandleHelper
	ScopeStrategy            fosite.ScopeStrategy
	AudienceMatchingStrategy fosite.AudienceMatchingStrategy
}

func (m *PersonalAccessTokenHandler) HandleTokenEndpointRequest(ctx context.Context, request fosite.AccessRequester) error {
	// grant_type REQUIRED.
	// Value MUST be set to "personal_access_token".
	if !request.GetGrantTypes().Exact("personal_access_token") {
		return errors.WithStack(fosite.ErrUnknownRequest)
	}

	if !request.GetClient().GetGrantTypes().Has("personal_access_token") {
		return errors.WithStack(fosite.ErrInvalidGrant.WithHint("The client is not allowed to use authorization grant \"personal_access_token\"."))
	}

	tkn := request.GetRequestForm().Get("access_token")
	sig := m.AccessTokenStrategy.AccessTokenSignature(tkn)
	or, err := m.AccessTokenStorage.GetAccessTokenSession(ctx, sig, request.GetSession())
	if errors.Cause(err) == fosite.ErrNotFound {
		return errors.WithStack(fosite.ErrInvalidRequest.WithDebug(err.Error()))
	} else if err != nil {
		return errors.WithStack(fosite.ErrServerError.WithDebug(err.Error()))
	} else if err := m.AccessTokenStrategy.ValidateAccessToken(ctx, or, tkn); err != nil {
		// The authorization server MUST ... validate the access token.
		// This needs to happen after store retrieval for the session to be hydrated properly
		rfce := fosite.ErrorToRFC6749Error(err)
		if rfce.Name == "error" {
			return errors.WithStack(fosite.ErrInvalidRequest.WithDebug(err.Error()))
		} else {
			return rfce
		}
	}

	if or.GetClient().GetID() != request.GetClient().GetID() {
		return errors.WithStack(fosite.ErrInvalidRequest.WithHint("The OAuth 2.0 Client ID from this request does not match the ID during the initial token issuance."))
	}

	request.SetSession(or.GetSession().Clone())

	for _, scope := range request.GetGrantedScopes() {
		if !m.ScopeStrategy(request.GetClient().GetScopes(), scope) {
			return errors.WithStack(fosite.ErrInvalidScope.WithHintf("The OAuth 2.0 Client is not allowed to request scope \"%s\".", scope))
		}
		request.GrantScope(scope)
	}

	if err := m.AudienceMatchingStrategy(request.GetClient().GetAudience(), or.GetGrantedAudience()); err != nil {
		return err
	}

	for _, audience := range request.GetGrantedAudience() {
		request.GrantAudience(audience)
	}

	// Personal Access Tokens never expire and don't need to be refreshed.
	// Since fosite does not allow non-expiring access tokens we set expiration to 100 years.
	request.GetSession().SetExpiresAt(fosite.AccessToken, time.Now().UTC().Add(m.AccessTokenLifespan).Round(time.Second))
	return nil
}

func (m *PersonalAccessTokenHandler) PopulateTokenEndpointResponse(ctx context.Context, requester fosite.AccessRequester, responder fosite.AccessResponder) error {
	if !requester.GetGrantTypes().Exact("personal_access_token") {
		return errors.WithStack(fosite.ErrUnknownRequest)
	}

	if err := m.IssueAccessToken(ctx, requester, responder); err != nil {
		return err
	}

	return nil
}
