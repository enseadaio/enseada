<template>
  <section class="section">
    <h1 class="title">Create new user</h1>
    <validation-observer v-slot="{ invalid, handleSubmit }">
      <form @submit.prevent="handleSubmit(submit)">
        <validation-provider rules="required"
                             name="username"
                             v-slot="{ errors, ...v }">
          <b-field label="Username"
                   :type="fieldType(v)"
                   :message="errors[0]">
            <b-input v-model="model.username"></b-input>
          </b-field>
        </validation-provider>
        <validation-provider rules="required|confirmed:confirmation"
                             vid="password"
                             name="password"
                             v-slot="{ errors, ...v }">
          <b-field label="Password"
                   :type="fieldType(v)"
                   :message="errors[0]">
            <b-input v-model="model.password"
                     password-reveal
                     type="password"></b-input>
          </b-field>
        </validation-provider>
        <validation-provider vid="confirmation"
                             name="confirmation"
                             rules="required|confirmed:password"
                             v-slot="{ errors, ...v }">
          <b-field label="Confirm password"
                   :type="fieldType(v)"
                   :message="errors[0]">
            <b-input v-model="model.confirmation"
                     password-reveal
                     type="password"></b-input>
          </b-field>
        </validation-provider>
        <b-field label="Roles">
          <b-taginput v-model="model.roles"
                      maxtags="10"
                      :data="filteredRoles"
                      type="is-info"
                      placeholder="Add a role"
                      @add="addRole"
                      @typing="filterRoles"
                      autocomplete></b-taginput>
        </b-field>
        <b-input type="submit"
                 custom-class="button is-primary"
                 :loading="loading"
                 :disabled="invalid"
                 value="Submit"></b-input>
      </form>
    </validation-observer>
  </section>
</template>

<script>
import { createForm, validatedForm } from '../../mixins'

export default {
  name: 'UsersCreateForm',
  mixins: [createForm({ name: 'user', service: 'users', mapId: ({ username }) => username }), validatedForm],
  data () {
    return {
      model: {
        username: '',
        password: '',
        confirmation: '',
        roles: []
      },
      rolesPages: [],
      filteredRoles: []
    }
  },
  computed: {
    roles () {
      return this.rolesPages.flatMap(({ items }) => items).map(({ role }) => role)
    }
  },
  methods: {
    addRole () {
      this.model.roles = this.model.roles.map((r) => r.toLowerCase().replaceAll(' ', '-'))
    },
    async fetchRoles (offset = 0) {
      this.loading = true
      const page = await this.$roles.list({ offset, limit: 100 })
      this.loading = false
      this.rolesPages = [...this.rolesPages, page]
      if (page.offset + page.limit < page.total) {
        return this.fetchRoles(offset + 100)
      }
    },
    filterRoles (text) {
      this.filteredRoles = this.roles.filter((option) => option.indexOf(text) >= 0)
    }
  },
  created () {
    this.fetchRoles()
        .then(() => {
          this.filteredRoles = this.roles
        })
        .catch((err) => this.$emit('error', err))
  }
}
</script>

<style scoped>

</style>