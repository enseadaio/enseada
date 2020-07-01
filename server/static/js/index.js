import '../scss/styles.scss'
import Turbolinks from 'turbolinks'

Turbolinks.start()

if (module.hot) {
  module.hot.dispose(function () {
    window.location.reload()
  })
}