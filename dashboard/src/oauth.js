function buildUrl(path) {
  return window.location.origin + path;
}

const scope = [
  'profile',
  'users:read',
  'users:manage',
  'oci:repos:read',
  'oci:repos:manage',
  'oci:repos:delete'
].join(' ');

export const settings = {
  authority: buildUrl('/.well-known/openid-configuration'),
  client_id: 'enseada',
  redirect_uri: buildUrl('/dashboard/auth/callback'),
  response_type: 'code',
  scope,
  loadUserInfo: true,
  automaticSilentRenew: true,
  metadata: {
    authorization_endpoint: buildUrl('/oauth/authorize'),
    token_endpoint: buildUrl('/oauth/token'),
    userinfo_endpoint: buildUrl('/api/v1beta1/users/me'),
    end_session_endpoint: buildUrl('/oauth/logout')
  }
};

export const listeners = {
  userLoaded: () => console.debug('OIDC user is loaded'),
  userUnloaded: () => console.debug('OIDC user is unloaded'),
  accessTokenExpiring: () => console.warn('Access token will expire'),
  accessTokenExpired: () => console.warn('Access token did expire'),
  silentRenewError: () => console.error('OIDC user is unloaded'),
  userSignedOut: () => console.debug('OIDC user is signed out'),
  oidcError: (payload) => console.error(`An error occured at ${payload.context}:`, payload.error)
};