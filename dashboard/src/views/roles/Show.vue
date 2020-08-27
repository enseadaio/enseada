<template>
  <b-loading :is-full-page="true" v-model="loading" v-if="loading"></b-loading>
  <section class="section" v-else>
    <h1 class="title">Role '{{ name }}'</h1>
    <div class="level">
      <div class="level-right">
        <router-link class="level-item button is-info" to="#">Edit</router-link>
        <a class="level-item button is-danger"
           @click="remove">Delete</a>
      </div>
    </div>
    <div class="level">
      <div class="level-left">
        <h2 class="subtitle is-3">Assigned permissions</h2>
      </div>
      <div class="level-right">
        <a class="level-item button is-info"
           @click="openPermissionModal">Add</a>
      </div>
    </div>

    <PermissionsTable :page="permissionsPage"
                      @page-change="onPermissionsPageChange"
                      @remove="removePermission">
    </PermissionsTable>
  </section>
</template>

<script>
import PermissionsTable from '../../components/PermissionsTable'
import { permissionsTable } from '../../mixins'

export default {
  name: 'RoleShow',
  components: { PermissionsTable },
  mixins: [permissionsTable({ service: 'roles' })],
  props: {
    id: String
  },
  data () {
    return {
      loading: false
    }
  },
  computed: {
    name () {
      return this.id
    }
  },
  methods: {
    async remove () {
      try {
        await this.$roles.remove(this.id)
        this.$buefy.notification.open({
          message: `Deleted ${name} ${this.id}`,
          type: 'is-warning',
          position: 'is-bottom-right',
          duration: 2000
        })

        return this.$router.push({ name: 'roles' })
      } catch (err) {
        return this.$emit('error', err)
      }
    }
  },
  created () {
    return this.$roles.exists(this.id).catch((err) => this.$emit('error', err))
  }
}
</script>

<style scoped>

</style>