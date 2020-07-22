<template>
    <section class="section">
        <h1 class="title">Create new user</h1>
        <form @submit.prevent="submit">
            <b-field label="Username">
                <b-input v-model="username"></b-input>
            </b-field>
            <b-field label="Password">
                <b-input v-model="password"
                         password-reveal
                         type="password"></b-input>
            </b-field>
            <b-field label="Roles">
                <b-taginput v-model="roles"
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
  export default {
    name: 'UsersCreateForm',
    data () {
      return {
        loading: false,
        username: '',
        password: '',
        roles: []
      }
    },
    methods: {
      async submit () {
        try {
          this.loading = true
          const { username } = await this.$users.create({
            username: this.username,
            password: this.password,
            roles: this.roles
          })
          this.$buefy.notification.open({
            message: `Created user ${username}`,
            type: 'is-success',
            position: 'is-bottom-right',
            duration: 10000
          })
          await this.$router.push({ name: 'users' })
        } catch (err) {
          this.$emit('error', err)
        } finally {
          this.loading = false
        }
      },
      addRole () {
        this.roles = this.roles.map((r) => r.toLowerCase().replaceAll(' ', '-'))
      }
    }
  }
</script>

<style scoped>

</style>