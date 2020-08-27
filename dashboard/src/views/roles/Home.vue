<template>
  <section class="section">
    <h1 class="title">Roles</h1>
    <div class="level">
      <div class="level-right">
        <router-link class="level-item button is-info" :to="{name: 'create-role'}">Create</router-link>
        <a class="level-item button is-danger"
           @click="remove"
           :disabled="!checked.length">Delete</a>
      </div>
    </div>
    <b-table :data="items"
             :total="count"
             :per-page="limit"
             :checked-rows.sync="checked"
             :loading="loading"
             @page-change="onPageChange"
             checkable
             backend-pagination
             paginated>
      <b-table-column field="role" label="Name" v-slot="{ row }">
        <router-link :to="{ name: 'role', params: { id: row.role }}">{{ row.role }}</router-link>
      </b-table-column>
    </b-table>
  </section>
</template>

<script>
import { listPage } from '../../mixins'

export default {
  name: 'RolesHome',
  mixins: [listPage({ name: 'role', service: 'roles', mapId: ({ role }) => role })]
}
</script>

<style scoped>

</style>