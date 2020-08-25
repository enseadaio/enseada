<template>
    <section class="section">
        <div class="level">
            <div class="level-left">
                <h1 class="level-item title is-4">There {{ count === 1 ? 'is' : 'are' }} {{ count }} {{ count === 1 ? 'user' : 'users' }} registered</h1>
            </div>
            <div class="level-right">
                <router-link class="level-item button is-primary" :to="{name: 'create-user'}">Create</router-link>
                <a class="level-item button is-danger"
                   @click="remove"
                   :disabled="!checked.length">Delete</a>
            </div>
        </div>
      <b-table :data="items"
               :columns="columns"
               :total="count"
               :per-page="limit"
               :checked-rows.sync="checked"
               @page-change="onPageChange"
               checkable
               :is-row-checkable="(row) => row.username !== 'root'"
               backend-pagination
               paginated></b-table>
    </section>
</template>

<script>
import { listPage } from '../../mixins'

export default {
  name: 'UsersHome',
  mixins: [listPage({ name: 'user', service: 'users', mapId: ({ username }) => username })],
  computed: {
    columns () {
      return [
        { field: 'username', label: 'Username' }
      ]
    }
  }
}
</script>

<style scoped>

</style>