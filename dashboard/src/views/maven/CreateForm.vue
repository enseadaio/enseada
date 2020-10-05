<template>
  <section class="section">
    <h1 class="title">Create new Maven repository</h1>
    <form @submit.prevent="submit">
      <b-field label="Group ID">
        <b-input v-model="model.group_id"></b-input>
      </b-field>
      <b-field label="Artifact ID">
        <b-input v-model="model.artifact_id"></b-input>
      </b-field>
      <b-field label="Public">
        <b-switch type="is-info"
                  v-model="model.public">
          {{ model.public ? 'Public' : 'Private' }}
        </b-switch>
      </b-field>
      <b-input :loading="loading"
               custom-class="button is-primary"
               type="submit"
               value="Submit"></b-input>
    </form>
  </section>
</template>

<script>
import { createForm } from '../../mixins'

export default {
  name: 'MavenCreateForm',
  mixins: [createForm({
    name: 'repository',
    service: 'maven',
    mapId: ({ group_id, artifact_id }) => `${group_id}/${artifact_id}`,
    permission: { object: 'maven_repos', action: 'manage' }
  })
  ],
  data () {
    return {
      model: {
        group_id: null,
        artifact_id: null,
        public: false
      }
    }
  }
}
</script>

<style scoped>

</style>