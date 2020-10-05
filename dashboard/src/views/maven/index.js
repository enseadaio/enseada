import Home from './Home'
import CreateForm from './CreateForm'
import Show from './Show'

const routes = [
  { path: '/maven', name: 'maven', component: Home },
  { path: '/maven/new', name: 'create-maven-repo', component: CreateForm },
  { path: '/maven/:group_id/:artifact_id', name: 'maven-repo', component: Show, props: true }
]

export { routes }
