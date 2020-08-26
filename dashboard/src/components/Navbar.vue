<template>
    <b-navbar>
        <template slot="brand">
            <b-navbar-item tag="router-link" :to="{ name: 'home' }">
                <img :src="enseadaLogo" height="50" width="50"/>
            </b-navbar-item>
        </template>
        <template slot="start">
            <b-navbar-item tag="router-link" :to="{ name: 'home' }">
                Home
            </b-navbar-item>
            <b-navbar-dropdown v-for="section of sections"
                               :key="section.name"
                               :label="section.name"
                               class="is-hidden-desktop"
                               hoverable>
                <b-navbar-item v-for="child of (section.children || [])"
                               :key="child.name"
                        tag="router-link" :to="child.to">
                    {{ child.name }}
                </b-navbar-item>
            </b-navbar-dropdown>
            <b-navbar-dropdown label="Help" hoverable>
                <b-navbar-item href="https://docs.enseada.io" target="_blank">
                    Documentation
                </b-navbar-item>
                <b-navbar-item href="https://github.com/enseadaio/enseada/issues/new/choose"
                   target="_blank">
                    Report an issue
                </b-navbar-item>
                <hr class="navbar-divider" />
                <b-navbar-item tag="router-link" :to="{ name: 'about' }">
                    About
                </b-navbar-item>
            </b-navbar-dropdown>
        </template>

        <template slot="end">
            <b-navbar-dropdown :label="username" hoverable>
              <b-navbar-item href="#">
                Account
              </b-navbar-item>
              <b-navbar-item @click="this.signOut">
                Logout
              </b-navbar-item>
            </b-navbar-dropdown>
        </template>
    </b-navbar>
</template>

<script>
import { mapActions, mapGetters } from 'vuex'
import enseadaLogo from '../../assets/images/enseada-logo.svg'
import sections from '../sections'

export default {
  name: 'Navbar',
  computed: {
    ...mapGetters(['currentUser']),
    sections: () => (sections),
    enseadaLogo: () => (enseadaLogo),
    username () {
      return this.currentUser.username
    }
  },
  methods: {
    ...mapActions(['authenticateOidc', 'signOutOidc']),
    signOut () {
      this.signOutOidc()
          .catch((e) => this.$emit('error', e))
    }
  }
  }
</script>

<style scoped>

</style>