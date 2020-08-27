import Home from './Home'
import CreateForm from './CreateForm'
import Show from './Show'

const routes = [
  { path: '/roles', name: 'roles', component: Home },
  { path: '/roles/new', name: 'create-role', component: CreateForm },
  { path: '/roles/:id', name: 'role', component: Show, props: true }
]

export { routes }
