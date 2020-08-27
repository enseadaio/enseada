<template>
  <b-loading :is-full-page="true" v-model="loading" v-if="loading"></b-loading>
  <section class="section" v-else>
    <h1 class="title">{{ model.username | titleCase }}</h1>
    <h2 class="subtitle">Roles</h2>
    <b-table :data="roles"
             :total="rolesPage.count"
             :per-page="rolesPage.limit"
             @page-change="onRolesPageChange"
             backend-pagination
             paginated>
      <b-table-column field="role" label="Name" v-slot="{ row }">
        <router-link :to="{ name: 'role', params: { id: row.role }}">{{ row.role }}</router-link>
      </b-table-column>
    </b-table>

    <PermissionsTable :page="permissionsPage"
                      @page-change="onPermissionsPageChange"
                      @remove="removePermission">
    </PermissionsTable>
  </section>
</template>

<script>
import showPage from '../../mixins/showPage'
import { pageToOffset } from '../../http'
import PermissionsTable from '../../components/PermissionsTable'
import { permissionsTable } from '../../mixins'

export default {
  name: 'UsersShow',
  components: { PermissionsTable },
  mixins: [showPage({ name: 'user', service: 'users' }), permissionsTable({ service: 'users' })],
  data () {
    return {
      model: null,
      rolesPage: {
        limit: 25
      }
    }
  },
  computed: {
    roles () {
      if (!this.rolesPage.items) return []
      return this.rolesPage.items.map((role) => ({ role }))
    },
    rolesSvc () {
      return this.$users.association('roles', this.id)
    }
  },
  methods: {
    async fetchRoles (offset = 0) {
      this.loading = true
      this.rolesPage = await this.rolesSvc.list({ offset, limit: this.rolesPage.limit })
      this.loading = false
    },
    onRolesPageChange (page) {
      return this.fetchRoles(pageToOffset(page, this.rolesPage.limit)).catch((err) => this.$emit('error', err))
    }
  },
  created () {
    return this.fetchRoles().catch((err) => this.$emit('error', err))
  }
}
</script>

<style scoped>

</style>