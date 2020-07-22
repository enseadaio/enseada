<template>
    <b-menu>
        <b-menu-list v-for="section of sections"
                           :key="section.name"
                           :label="section.name">
            <b-menu-item v-for="child of (section.children || [])"
                           :key="child.name"
                           :label="child.name"
                           tag="router-link" :to="child.to">
            </b-menu-item>
        </b-menu-list>
    </b-menu>
</template>

<script>
  import sections from '../sections';

  export default {
    name: 'SideMenu',
    computed: {
      sections: () => (sections),
    },
  }
</script>

<style scoped>

</style>