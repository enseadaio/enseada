<template>
  <section class="section">
    <h1 class="title">Containers</h1>
    <div class="level">
      <div class="level-right">
        <router-link :to="{name: 'create-maven-repo'}"
                     class="level-item button info"
                     v-if="check('maven_repos', 'create')">Create
        </router-link>
        <a :disabled="!checked.length"
           @click="remove"
           class="level-item button is-danger"
           v-if="check('oci_repos', 'delete')">Delete</a>
      </div>
    </div>
    <b-table :checked-rows.sync="checked"
             :columns="columns"
             :data="items"
             :loading="loading"
             :per-page="limit"
             :total="count"
             @page-change="onPageChange"
             backend-pagination
             checkable
             paginated></b-table>
  </section>
</template>

<script>
import { listPage } from '../../mixins'

export default {
  name: 'MavenHome',
  mixins: [listPage({
    name: 'repository',
    service: 'maven',
    mapId: ({ group_id, artifact_id }) => `${group_id}/${artifact_id}`,
    permission: { object: 'maven_repos', action: 'read' }
  })
  ],
  computed: {
    items () {
      return this.page.items
    },
    columns () {
      return [
        { field: 'group_id', label: 'Group ID' },
        { field: 'artifact_id', label: 'Artifact ID' },
        { field: 'public', label: 'Public' }
      ]
    }
  }
}
</script>

<style scoped>

</style>