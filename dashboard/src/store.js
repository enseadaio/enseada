import Vue from 'vue';
import Vuex from 'vuex';
import { vuexOidcCreateStoreModule } from 'vuex-oidc';
import { settings, listeners } from './oauth';

Vue.use(Vuex);

const store = new Vuex.Store({
  modules: {
    oidcStore: vuexOidcCreateStoreModule(settings, {}, listeners),
  },
  state: {},
  getters: {},
  mutations: {},
  actions: {},
})

export function accessTokenProvider() {
  return Promise.resolve(store.state.oidcStore.access_token);
}

export default store;