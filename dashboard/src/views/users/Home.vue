<template>
    <section class="section">
        <div class="level">
            <div class="level-left">
                <h1 class="level-item title is-4">There {{ count === 1 ? 'is' : 'are' }} {{ count }} {{ count === 1 ? 'user' : 'users' }} registered</h1>
            </div>
            <div class="level-right">
                <router-link class="level-item button is-primary" :to="{name: 'create-user'}">Create</router-link>
                <a class="level-item button is-danger"
                        @click="deleteUsers"
                        :disabled="!checked.length">Delete</a>
            </div>
        </div>
        <b-table :data="page.items"
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
  import { pageToOffset } from '../../http'

  export default {
    name: 'UsersHome',
    data() {
      return {
        limit: 25,
        page: {
          count: 0,
          total: 0,
          offset: 0,
          limit: 0,
          items: [],
        },
        checked: [],
        loading: true,
      };
    },
    computed: {
      count() {
        return this.page.count;
      },
      columns() {
        return [
          { field: 'username', label: 'Username' }
        ];
      }
    },
    methods: {
      async fetchUsers (offset = 0) {
        this.loading = true;
        this.page = await this.$users.list({ offset, limit: this.limit });
      },
      async onPageChange(page) {
        await this.fetchUsers(pageToOffset(page, this.limit))
      },
      async deleteUsers() {
        const p = this.checked.map(({ username }) => this.$users.delete(username))
        const results = await Promise.all(p);
        results.forEach((res) => {
          console.log('Deleted user', res);
        })
      },
    },
    created () {
      this.fetchUsers().catch((err) => this.$emit('error', err))
    }
  }
</script>

<style scoped>

</style>