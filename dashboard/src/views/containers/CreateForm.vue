<template>
  <section class="section">
    <h1 class="title">Create new container repository</h1>
    <form @submit.prevent="submit">
      <b-field label="Group">
        <b-input v-model="model.group"></b-input>
      </b-field>
      <b-field label="Name">
        <b-input v-model="model.name"></b-input>
      </b-field>
      <b-field label="Description">
        <b-input v-model="model.description"></b-input>
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
  name: 'ContainersCreateForm',
  mixins: [createForm({ name: 'repository', service: 'containers', mapId: ({ group, name }) => `${group}/${name}` })],
  data () {
    return {
      model: {
        group: null,
        name: null,
        description: null
      }
    }
  }
}
</script>

<style scoped>

</style>