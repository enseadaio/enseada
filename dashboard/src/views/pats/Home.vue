<template>
  <section class="section">
    <h1 class="title">Personal Access Tokens</h1>
    <div class="level">
      <div class="level-right">
        <router-link class="level-item button info" :to="{name: 'create-pat'}">Create</router-link>
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
             :loading="loading"
             @page-change="onPageChange"
             checkable
             backend-pagination
             paginated></b-table>
  </section>
</template>

<script>
import { listPage } from '../../mixins'
import { mapId } from '../../utils'

export default {
  name: 'PersonalAccessTokensHome',
  mixins: [listPage({
    name: 'personal access token',
    service: 'pats',
    mapId
  })],
  computed: {
    items () {
      return this.page.items.map((pat) => ({ ...pat, fullName: this.mapId(pat) }))
    },
    columns () {
      return [
        { field: 'label', label: 'Label' },
        { field: 'client_id', label: 'Client ID' },
        { field: 'scope', label: 'Scope' },
        { field: 'expiration', label: 'Expiration' }
      ]
    }
  }
}
</script>

<style scoped>

</style>