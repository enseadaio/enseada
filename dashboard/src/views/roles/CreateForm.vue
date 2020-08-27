<template>
  <section class="section">
    <h1 class="title">Create new role</h1>
    <form @submit.prevent="submit">
      <b-field label="Name">
        <b-input v-model="model.name"></b-input>
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
  name: 'RolesCreateForm',
  mixins: [createForm({ name: 'role', service: 'roles', mapId: ({ name }) => name })],
  data () {
    return {
      model: {
        name: null
      }
    }
  }
}
</script>

<style scoped>

</style>