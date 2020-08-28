<template>
  <section class="section">
    <h1 class="title">Create new role</h1>
    <validation-observer v-slot="{ invalid, handleSubmit }">
      <form @submit.prevent="handleSubmit(submit)">
        <validation-provider rules="required"
                             name="name"
                             v-slot="{ errors, ...v }">
          <b-field label="Name"
                   :type="fieldType(v)"
                   :message="errors[0]">
            <b-input v-model="model.name"></b-input>
          </b-field>
        </validation-provider>
        <br>
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
  name: 'RolesCreateForm',
  mixins: [createForm({ name: 'role', service: 'roles', mapId: ({ name }) => name }), validatedForm],
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