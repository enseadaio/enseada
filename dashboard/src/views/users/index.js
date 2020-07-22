import Home from './Home';
import CreateForm from './CreateForm'
const routes = [
  { path: '/users', name: 'users', component: Home, },
  { path: '/users/new', name: 'create-user', component: CreateForm, },
];

export { routes };
