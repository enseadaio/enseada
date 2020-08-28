<template>
  <validation-observer v-slot="{ invalid, handleSubmit }">
    <form @submit.prevent="handleSubmit(submit)">
      <div class="modal-card" style="width: auto">
        <header class="modal-card-head">
          <p class="modal-card-title">Add permission</p>
        </header>
        <section class="modal-card-body">
          <validation-provider name="subject"
                               rules="required"
                               v-slot="{ errors, ...v }"
                               v-if="includeSubject">
            <b-field label="Subject">
              <b-input v-model="model.subject"
                       placeholder="user:monster"
                       :type="fieldType(v)"
                       :message="errors[0]"
                       required>
              </b-input>
            </b-field>
          </validation-provider>

          <validation-provider name="object"
                               rules="required"
                               v-slot="{ errors, ...v }">
            <b-field label="Object">
              <b-input v-model="model.object"
                       placeholder="cookies:*"
                       required>
              </b-input>
            </b-field>
          </validation-provider>

          <validation-provider name="action"
                               rules="required"
                               v-slot="{ errors, ...v }">
            <b-field label="Action">
              <b-input v-model="model.action"
                       placeholder="eat"
                       required>
              </b-input>
            </b-field>
          </validation-provider>

        </section>
        <footer class="modal-card-foot">
          <button class="button" type="button" @click="$emit('close')">Close</button>
          <b-input type="submit"
                   custom-class="button is-info"
                   :disabled="invalid"
                   value="Add"></b-input>
        </footer>
      </div>
    </form>
  </validation-observer>
</template>

<script>
import { validatedForm } from '../mixins'
import { ValidationObserver, ValidationProvider } from 'vee-validate'

export default {
  name: 'AddPermissionModal',
  mixins: [validatedForm],
  components: { ValidationObserver, ValidationProvider },
  props: {
    includeSubject: {
      type: Boolean,
      default: true
    }
  },
  data () {
    return {
      model: {
        subject: null,
        object: null,
        action: null
      }
    }
  },
  methods: {
    submit () {
      this.$emit('ok', this.model)
      this.$emit('close')
    }
  }
}
</script>

<style scoped>

</style>