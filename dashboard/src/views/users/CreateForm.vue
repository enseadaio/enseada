<template>
  <section class="section">
    <h1 class="title">Create new user</h1>
    <form @submit.prevent="submit">
      <b-field label="Username">
        <b-input v-model="model.username"></b-input>
      </b-field>
      <b-field label="Password">
        <b-input v-model="model.password"
                 password-reveal
                 type="password"></b-input>
      </b-field>
      <b-field label="Roles">
        <b-taginput v-model="model.roles"
                    maxtags="10"
                    @add="addRole"
                    type="is-info"
                    placeholder="Add a role"></b-taginput>
      </b-field>
      <b-input type="submit"
               custom-class="button is-primary"
               :loading="loading"
               value="Submit"></b-input>
    </form>
  </section>
</template>

<script>
import { createForm } from '../../mixins'

export default {
  name: 'UsersCreateForm',
  mixins: [createForm({ name: 'user', service: 'users', mapId: ({ username }) => username })],
  data () {
    return {
      model: {
        username: '',
        password: '',
        roles: []
      }
    }
  },
  methods: {
    addRole () {
      this.model.roles = this.model.roles.map((r) => r.toLowerCase().replaceAll(' ', '-'))
    }
  }
}
</script>

<style scoped>

</style>