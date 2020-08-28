import Vue from 'vue';
import Vuex, { StoreOptions } from 'vuex';
import { vuexOidcCreateStoreModule } from 'vuex-oidc';
import { listeners, settings } from '../oauth';
import { RootState } from "./types";
import { Capabilities, User } from "../types";

Vue.use(Vuex);

const opts: StoreOptions<RootState> = {
  state: () => ({ currentUser: null, capabilities: null }),
  getters: {
    currentUser: ({ currentUser }) => currentUser,
    capabilities: ({ capabilities }) => capabilities,
    permissions: ({ capabilities }) => capabilities?.permissions,
    ready: ({ currentUser, capabilities }) => (!!currentUser && !!capabilities),
  },
  mutations: {
    setCurrentUser(state: RootState, user: User) {
      state.currentUser = user;
    },
    removeCurrentUser(state: RootState) {
      state.currentUser = null;
    },
    setCapabilities(state: RootState, capabilities: Capabilities) {
      state.capabilities = capabilities;
    },
    removeCapabilities(state: RootState) {
      state.capabilities = null;
    },
  },
  actions: {
    async storeCurrentUser({ commit }) {
      const user = await this._vm.$users.get('me');
      commit('setCurrentUser', user);
    },
    removeCurrentUser({ commit }) {
      commit('removeCurrentUser')
    },
    async storeCapabilities({ commit }) {
      const caps = await this._vm.$users.get('me/capabilities');
      commit('setCapabilities', caps);
    },
    removeCapabilities({ commit }) {
      commit('removeCapabilities')
    }
  },
};
const store = new Vuex.Store<RootState>(opts)

store.registerModule('oidcStore', vuexOidcCreateStoreModule(settings, {}, listeners(store)))

export function accessTokenProvider() {
  return Promise.resolve(store.state.oidcStore.access_token);
}

export default store;