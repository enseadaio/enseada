<template>
    <section class="section">
        <b-loading :active="!error"></b-loading>
        <p v-if="!!error">{{ error }}</p>
    </section>
</template>


<script>
import { mapActions } from 'vuex'

export default {
  name: 'OAuthCallback',
  data () {
    return {
      error: null
    }
  },
  methods: {
    ...mapActions(['oidcSignInCallback', 'storeCurrentUser'])
  },
  mounted () {
    this.oidcSignInCallback()
        .then(async (redirect) => {
          await this.storeCurrentUser()
          return this.$router.push(redirect)
        })
        .catch((e) => {
          this.error = e
        })
    }
  }
</script>

<style scoped>

</style>