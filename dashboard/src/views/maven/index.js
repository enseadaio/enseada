import Home from './Home'
import CreateForm from './CreateForm'

const routes = [
  { path: '/maven', name: 'maven', component: Home },
  { path: '/maven/new', name: 'create-maven-repo', component: CreateForm }
]

export { routes }
