<template>
  <section class="section">
    <h1 class="title">Containers</h1>
    <div class="level">
      <div class="level-right">
        <router-link class="level-item button info"
                     v-if="check('oci_repos', 'create')"
                     :to="{name: 'create-container-repo'}">Create
        </router-link>
        <a class="level-item button is-danger"
           @click="remove"
           v-if="check('oci_repos', 'delete')"
           :disabled="!checked.length">Delete</a>
      </div>
    </div>
    <b-table :data="items"
             :columns="columns"
             :total="count"
             :per-page="limit"
             :checked-rows.sync="checked"
             :loading="loading"
             @page-change="onPageChange"
             checkable
             backend-pagination
             paginated></b-table>
  </section>
</template>

<script>
import { listPage } from '../../mixins'

export default {
  name: 'ContainersHome',
  mixins: [listPage({
    name: 'repository',
    service: 'containers',
    mapId: ({ group, name }) => `${group}/${name}`,
    permission: { object: 'oci_repos', action: 'read' }
  })
  ],
  computed: {
    items () {
      return this.page.items.map((repo) => ({ ...repo, fullName: this.mapId(repo) }))
    },
    columns () {
      return [
        { field: 'fullName', label: 'Name' }
      ]
    }
  }
}
</script>

<style scoped>

</style>