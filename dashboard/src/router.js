import Vue from 'vue'
import Router from 'vue-router'
import Home from './views/Home'
import { routes as usersRoutes } from './views/users'
import { routes as patsRoutes } from './views/pats'
import { routes as containersRoutes } from './views/containers'
import { routes as mavenRoutes } from './views/maven'
import About from './views/About'
import OAuthCallback from './views/OAuthCallback'
import { vuexOidcCreateRouterMiddleware } from 'vuex-oidc'
import store from './store'

Vue.use(Router);

const router = new Router({
  mode: 'history',
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/about', name: 'about', component: About },
    { path: '/dashboard/auth/callback', name: 'oauthCallback', component: OAuthCallback },
    ...usersRoutes,
    ...patsRoutes,
    ...containersRoutes,
    ...mavenRoutes
  ],
});

router.beforeEach(vuexOidcCreateRouterMiddleware(store));

export default router;
