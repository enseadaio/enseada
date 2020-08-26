import Vue from 'vue';
import Vuex, { StoreOptions } from 'vuex';
import { vuexOidcCreateStoreModule } from 'vuex-oidc';
import { listeners, settings } from '../oauth';
import { RootState } from "./types";
import { User } from "../types";

Vue.use(Vuex);

const opts: StoreOptions<RootState> = {
  state: () => ({ currentUser: null }),
  getters: {
    currentUser({ currentUser }) {
      return currentUser
    },
  },
  mutations: {
    setCurrentUser(state: RootState, user: User) {
      state.currentUser = user;
    },
    removeCurrentUser(state: RootState) {
      state.currentUser = null;
    }
  },
  actions: {
    async storeCurrentUser({ commit }) {
      const localUser = localStorage.getItem('currentUser');
      if (localUser) {
        commit('setCurrentUser', JSON.parse(localUser));
      } else {
        const user = await this._vm.$users.get('me');
        localStorage.setItem('currentUser', JSON.stringify(user));
        commit('setCurrentUser', user);
      }
    },
    removeCurrentUser({ commit }) {
      localStorage.removeItem('currentUser');
      commit('removeCurrentUser')
    }
  },
};
const store = new Vuex.Store<RootState>(opts)

store.registerModule('oidcStore', vuexOidcCreateStoreModule(settings, {}, listeners(store)))

export function accessTokenProvider() {
  return Promise.resolve(store.state.oidcStore.access_token);
}

export default store;