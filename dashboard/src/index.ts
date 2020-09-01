import Vue from 'vue';
import Buefy from 'buefy';
import VueHttp from './http';
import App from './App.vue';
import router from './router';
import store, { accessTokenProvider } from './store';
import { i18n } from './i18n';

import './filters';
import './validations';

import '@fortawesome/fontawesome-free/css/all.css';
import '../assets/scss/style.scss';
import 'buefy/dist/buefy.css';

if (module.hot) {
  module.hot.accept();
}

window.onerror = function (message, file, line, col, error) {
  alert("Error occurred: " + error.message);
  return false;
};
window.addEventListener("error", function (e) {
  alert("Error occurred: " + e.error.message);
  return false;
})

Vue.use(Buefy, {
  defaultProgrammaticPromise: true
});
Vue.use(VueHttp, { accessTokenProvider });

new Vue({
  router,
  store,
  i18n,
  render: (h) => h(App)
}).$mount('#app');