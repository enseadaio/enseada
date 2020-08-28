<template>
  <b-menu>
    <b-menu-list v-for="section of sections"
                 :key="section.name"
                 :label="section.name">
      <b-menu-item v-for="child of (section.children || [])"
                   :key="child.name"
                   :label="child.name"
                   tag="router-link"
                   :to="child.to"
                   v-if="!child.permission || check(child.permission.object, child.permission.action)">
      </b-menu-item>
    </b-menu-list>
  </b-menu>
</template>

<script>
import sections from '../sections'
import { check } from '../auth'
import { mapGetters } from 'vuex'

export default {
  name: 'SideMenu',
  computed: {
    ...mapGetters(['currentUser', 'permissions']),
    sections: () => (sections)
  },
  methods: {
    check
  }
}
</script>

<style scoped>

</style>