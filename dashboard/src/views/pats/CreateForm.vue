<template>
  <section class="section">
    <h1 class="title">Create new personal access token</h1>
    <form @submit.prevent="submit">
      <b-field label="Label">
        <b-input v-model="model.label"></b-input>
      </b-field>
      <h3 class="subtitle is-6"><strong>Scope</strong></h3>
      <div class="columns is-multiline">
        <div class="column box is-one-quarter" v-for="[label, scopeList] of availableScopes">
          <h4 class="subtitle is-8"><strong>{{ label }}</strong></h4>
          <b-checkbox v-model="scopes"
                      @input="addScope"
                      v-for="scope of scopeList" :key="scope"
                      :native-value="scope">
            {{ scope }}
          </b-checkbox>
        </div>
      </div>
      <b-field label="Expiration">
        <b-datetimepicker
            rounded
            placeholder="Click to select..."
            icon="calendar-alt"
            icon-pack="fas"
            v-model="model.expiration"
            :min-datetime="new Date()"
            horizontal-time-picker>
        </b-datetimepicker>
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
import { mapId } from '../../utils'
import { SCOPES } from '../../oauth'

export default {
  name: 'PersonalAccessTokensCreateForm',
  mixins: [createForm({
    name: 'personal access token',
    service: 'pats',
    mapId
  })],
  data () {
    return {
      scopes: [],
      model: {
        label: null,
        scope: null,
        expiration: null
      }
    }
  },
  computed: {
    availableScopes () {
      return Object.entries(SCOPES)
    }
  },
  methods: {
    postSubmit () {
      this.$buefy.dialog.alert({
        title: `Created personal access token ${this.created.label}`,
        message: `
         <strong>${this.created.access_token}</strong><br><br>

        Make sure to copy your new personal access token now. You won’t be able to see it again!
        `,
        type: 'is-info',
        size: 'is-medium'
      })
      return this.$router.push({ name: 'pats' })
    },
    addScope () {
      this.model.scope = this.scopes.map((r) => r.toLowerCase().replaceAll(' ', '-')).join(' ')
    }
  }
}
</script>

<style scoped>

</style>