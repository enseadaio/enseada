<template>
  <b-navbar>
    <template slot="brand">
      <b-navbar-item tag="router-link" :to="{ name: 'home' }">
        <img :src="enseadaLogo" height="50" width="50"/>
      </b-navbar-item>
    </template>
    <template slot="start">
      <b-navbar-item tag="router-link" :to="{ name: 'home' }">
        {{ $t('home') | titleCase }}
      </b-navbar-item>
      <b-navbar-dropdown v-for="section of sections"
                         :key="section.name"
                         :label="$t(section.name) | pascalCase"
                         class="is-hidden-desktop"
                         hoverable>
        <b-navbar-item v-for="child of (section.children || [])"
                       :key="child.name"
                       tag="router-link"
                       v-if="!child.permission || check(child.permission.object, child.permission.action)"
                       :to="child.to">
          {{ $t(child.name) | pascalCase }}
        </b-navbar-item>
      </b-navbar-dropdown>
      <b-navbar-dropdown label="Help" hoverable>
        <b-navbar-item href="https://docs.enseada.io" target="_blank">
          {{ $t('documentation') | titleCase }}
        </b-navbar-item>
        <b-navbar-item href="/api/docs" target="_blank">
          {{ $t('apiDocs') | pascalCase }}
        </b-navbar-item>
        <b-navbar-item href="https://github.com/enseadaio/enseada/issues/new/choose"
                       target="_blank">
          {{ $t('issueReport') | titleCase }}
        </b-navbar-item>
        <hr class="navbar-divider"/>
        <b-navbar-item tag="router-link" :to="{ name: 'about' }">
          {{ $t('about') | titleCase }}
        </b-navbar-item>
      </b-navbar-dropdown>
    </template>

    <template slot="end">
      <b-navbar-dropdown :label="username | titleCase" hoverable>
        <b-navbar-item href="#">
          {{ $t('account') | titleCase }}
        </b-navbar-item>
        <b-navbar-item @click="this.signOut">
          {{ $t('logout') | titleCase }}
        </b-navbar-item>
      </b-navbar-dropdown>
    </template>
  </b-navbar>
</template>

<script>
import { mapActions, mapGetters } from 'vuex'
import enseadaLogo from '../../assets/images/enseada-logo.svg'
import sections from '../sections'
import { check } from '../auth'

export default {
  name: 'Navbar',
  computed: {
    ...mapGetters(['currentUser', 'permissions']),
    sections: () => (sections),
    enseadaLogo: () => (enseadaLogo),
    username () {
      if (!this.currentUser) {
        return 'loading...'
      }

      return this.currentUser.username
    }
  },
  methods: {
    ...mapActions(['authenticateOidc', 'signOutOidc']),
    check,
    signOut () {
      this.signOutOidc()
          .catch((e) => this.$emit('error', e))
    }
  }
}
</script>

<style scoped>

</style>