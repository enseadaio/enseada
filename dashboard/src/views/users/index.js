import Home from './Home'
import CreateForm from './CreateForm'
import Show from './Show'

const routes = [
  { path: '/users', name: 'users', component: Home },
  { path: '/users/new', name: 'create-user', component: CreateForm },
  { path: '/users/:id', name: 'user', component: Show, props: true }
]

export { routes }
