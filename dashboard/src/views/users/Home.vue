<template>
  <section class="section">
    <div class="level">
      <div class="level-left">
        <h1 class="level-item title is-4">There {{ count === 1 ? 'is' : 'are' }} {{ count }}
          {{ count === 1 ? 'user' : 'users' }} registered</h1>
      </div>
      <div class="level-right">
        <router-link class="level-item button is-info" :to="{name: 'create-user'}">Create</router-link>
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
             :is-row-checkable="isNotRoot"
             @page-change="onPageChange"
             checkable
             backend-pagination
             paginated>
      <b-table-column field="username" label="Username" v-slot="{ row }">
        <router-link :to="{ name: 'user', params: { id: row.username }}" v-if="isNotRoot(row)">{{ row.username }}
        </router-link>
        <span v-else>{{ row.username }}</span>
      </b-table-column>
      <b-table-column label="Status" v-slot="{ row: { username, enabled } }">
        <b-tooltip label="Root user cannot be disabled"
                   position="is-left"
                   type="is-info"
                   v-if="username === 'root'">
          <b-switch :value="enabled"
                    type="is-success"
                    passive-type="is-danger"
                    disabled>
            {{ enableText(enabled) }}
          </b-switch>
        </b-tooltip>
        <b-switch :value="enabled"
                  type="is-success"
                  passive-type="is-danger"
                  @click.native.prevent="toggle(username, enabled)"
                  v-else>
          {{ enableText(enabled) }}
        </b-switch>
      </b-table-column>
    </b-table>
  </section>
</template>

<script>
import { listPage } from '../../mixins'

export default {
  name: 'UsersHome',
  mixins: [listPage({ name: 'user', service: 'users', mapId: ({ username }) => username })],
  methods: {
    enableText (enabled) {
      return enabled ? 'Enabled' : 'Disabled'
    },
    isNotRoot ({ username }) {
      return username !== 'root'
    },
    async toggle (username, enabled) {
      try {
        const action = enabled ? 'disabled' : 'enabled'
        const { result, dialog } = await this.$buefy.dialog.confirm({
          message: `User '${username}' is going to be ${action}. Are you sure?`,
          closeOnConfirm: false
        })
        if (result) {
          await this.$users.update(username, { enabled: !enabled })
          await this.reloadCurrent()
          dialog.close()
          return this.$buefy.notification.open({
            message: `User '${username}' has been ${action}`,
            type: 'is-success',
            position: 'is-bottom-right',
            duration: 2000
          })
        }
      } catch (err) {
        this.$emit('error', err)
      }
    }
  }
}
</script>

<style scoped>

</style>