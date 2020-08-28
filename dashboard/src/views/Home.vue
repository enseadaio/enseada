<template>
  <section class="section">
    <h1 class="title">Dashboard</h1>
    <div class="columns is-multiline">
      <div class="column">
        <PackageCard :to="{ name: 'containers' }"
                     :image="ociLogo"
                     bg-color="#2396ec"
                     v-if="check('oci_repos', 'read')"
                     title="Container Images">
          {{ containersCount }} available
        </PackageCard>
      </div>
      <div class="column">
        <PackageCard :to="{ name: 'maven' }"
                     :image="mavenLogo"
                     bg-color="#ed6712"
                     v-if="check('maven_repos', 'read')"
                     title="Maven packages">
          {{ mavenCount }} available
        </PackageCard>
      </div>
    </div>
  </section>
</template>

<script>
import { mapGetters } from 'vuex'
import ociLogo from '../../assets/images/oci-logo.png'
import mavenLogo from '../../assets/images/maven-logo.png'
import PackageCard from '../components/PackageCard'
import { check } from '../auth'

export default {
  name: 'Home',
  components: { PackageCard },
  data () {
    return {
      containersCount: 0,
      mavenCount: 0
    }
  },
  computed: {
    ...mapGetters(['currentUser', 'permissions']),
    ociLogo: () => (ociLogo),
    mavenLogo: () => (mavenLogo)
  },
  methods: {
    check,
    async fetchContainersCount () {
      const { count } = await this.$containers.list()
      this.containersCount = count
    }
  },
  mounted () {
    if (this.check('oci_repos', 'read')) {
      this.fetchContainersCount().catch((err) => this.$emit('error', err))
    }
  }
}
</script>

<style scoped>

</style>