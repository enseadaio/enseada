import { Store } from 'vuex';
import { VuexOidcClientSettings } from 'vuex-oidc'
import { User as OidcUser } from "oidc-client";
import { RootState } from "./store/types";

function buildUrl(path?: string) {
  const origin = window.location.origin
  if (!path) {
    return origin
  }
  return origin + path
}

export const SCOPES = {
  users: [
    'profile',
    'users:read',
    'users:manage',
  ],
  rbac: [
    'roles',
    'permissions',
  ],
  tokens: [
    'pats:read',
    'pats:manage',
  ],
  oauth: [
    'clients:read',
    'clients:manage',
  ],
  containers: [
    'oci:repos:read',
    'oci:repos:manage',
    'oci:repos:delete',
    'oci:image:push',
    'oci:image:pull',
  ],
  maven: [
    'maven:repos:read',
    'maven:repos:manage',
    'maven:repos:delete',
  ],
};

const scope = Object.values(SCOPES).flat().join(' ');

export const settings: VuexOidcClientSettings = {
  authority: buildUrl(),
  metadataUrl: buildUrl('/.well-known/oauth-authorization-server'),
  clientId: 'enseada',
  redirectUri: buildUrl('/dashboard/auth/callback'),
  responseType: 'code',
  scope,
  automaticSilentRenew: true,
}

export const listeners = (store: Store<RootState>) => ({
  userLoaded: (_user: OidcUser) => {
    console.debug('OIDC user is loaded');
    store.dispatch('storeCurrentUser').catch()
  },
  userUnloaded: () => console.debug('OIDC user is unloaded'),
  accessTokenExpiring: () => console.warn('Access token will expire'),
  accessTokenExpired: () => console.warn('Access token did expire'),
  silentRenewError: () => console.error('OIDC user is unloaded'),
  userSignedOut: () => {
    console.debug('OIDC user is signed out');
    localStorage.removeItem('currentUser');
  },
  oidcError: (payload) => console.error(`An error occured at ${payload.context}:`, payload.error)
})
