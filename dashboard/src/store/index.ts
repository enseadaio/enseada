import Vue from 'vue';
import Vuex from 'vuex';
import { vuexOidcCreateStoreModule } from 'vuex-oidc';
import { listeners, settings } from '../oauth';
import containers from './containers';

Vue.use(Vuex);

const store = new Vuex.Store({
  modules: {
    oidcStore: vuexOidcCreateStoreModule(settings, {}, listeners),
    containers,
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