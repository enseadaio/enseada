import Home from './Home'
import CreateForm from './CreateForm'

const routes = [
  { path: '/containers', name: 'containers', component: Home },
  { path: '/containers/new', name: 'create-container-repo', component: CreateForm }
]

export { routes }
