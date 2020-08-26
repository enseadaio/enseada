import Home from './Home'
import CreateForm from './CreateForm'

const routes = [
  { path: '/access-tokens', name: 'pats', component: Home },
  { path: '/access-tokens/new', name: 'create-pat', component: CreateForm }
]

export { routes }
