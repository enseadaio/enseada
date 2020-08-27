<template>
  <div>
    <header>
      <Navbar @error="onError"/>
    </header>
    <main class="columns is-fullheight">
      <aside class="section column is-2 is-fullheight is-hidden-touch">
        <SideMenu/>
      </aside>
      <div class="section column">
        <NotFound v-if="notFound" :reasons="notFound"></NotFound>
        <router-view @error="onError" v-else></router-view>
      </div>
    </main>
  </div>
</template>

<script>
import SideMenu from './components/SideMenu'
import Navbar from './components/Navbar'
import NotFound from './views/NotFound'

export default {
  name: 'App',
  components: { SideMenu, Navbar, NotFound },
  data () {
    return {
      error: null,
      notFound: null
    }
  },
  watch: {
    $route () {
      this.notFound = null
    }
  },
  methods: {
    async onError (err) {
      console.error(err)
      console.log(err.response)
      if (err.response) {
        try {
          const { error, reasons } = await err.response.json()
          reasons.forEach((reason) => {
            this.$buefy.notification.open({
              type: 'is-danger',
              message: `${error}: ${reason}`,
              position: 'is-bottom-right',
              duration: 10000
            })
          })
          if (err.response.status === 404) {
            this.notFound = reasons
          }
        } catch (e) {
          if (!(e instanceof SyntaxError)) throw e
          if (err.response.status === 404) {
            this.notFound = []
          }
        }

      } else {
        this.$buefy.notification.open({
          type: 'is-danger',
          message: err.toString(),
          position: 'is-bottom-right',
          duration: 10000
        })
      }
    }
  }
}
</script>

<style scoped>

</style>