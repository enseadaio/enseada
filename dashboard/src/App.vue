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
                <router-view @error="onError"></router-view>
            </div>
        </main>
    </div>
</template>

<script>
  import { mapGetters } from 'vuex'
  import SideMenu from './components/SideMenu'
  import Navbar from './components/Navbar'

  export default {
    name: 'App',
    components: { SideMenu, Navbar },
    data () {
      return {
        error: null
      }
    },
    computed: {
      ...mapGetters([
        'oidcIsAuthenticated'
      ])
    },
    methods: {
      async onError (err) {
        console.error(err);
        if (err.response) {
            const { error, reasons } = await err.response.json();
            reasons.forEach((reason) => {
              this.$buefy.notification.open({
                type: 'is-danger',
                message: `${error}: ${reason}`,
                position: 'is-bottom-right',
                duration: 10000
              })
            });
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