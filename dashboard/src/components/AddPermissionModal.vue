<template>
  <form @submit.prevent="submit">
    <div class="modal-card" style="width: auto">
      <header class="modal-card-head">
        <p class="modal-card-title">Add permission</p>
      </header>
      <section class="modal-card-body">
        <b-field label="Subject" v-if="includeSubject">
          <b-input v-model="model.subject"
                   placeholder="user:monster"
                   required>
          </b-input>
        </b-field>

        <b-field label="Object">
          <b-input v-model="model.object"
                   placeholder="cookies:*"
                   required>
          </b-input>
        </b-field>

        <b-field label="Action">
          <b-input v-model="model.action"
                   placeholder="eat"
                   required>
          </b-input>
        </b-field>
      </section>
      <footer class="modal-card-foot">
        <button class="button" type="button" @click="$emit('close')">Close</button>
        <b-input type="submit"
                 custom-class="button is-info"
                 value="Add"></b-input>
      </footer>
    </div>
  </form>
</template>

<script>
export default {
  name: 'AddPermissionModal',
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