<template>
    <section class="section">
        <h1 class="title">Dashboard</h1>
        <div class="columns is-multiline">
            <div class="column">
                <PackageCard :to="{ name: 'containers' }"
                             :image="ociLogo"
                             bg-color="#2396ec"
                             title="Container Images">
                    {{ containersCount }} available
                </PackageCard>
            </div>
            <div class="column">
                <PackageCard :to="{ name: 'maven' }"
                             :image="mavenLogo"
                             bg-color="#ed6712"
                             title="Maven packages">
                    {{ mavenCount }} available
                </PackageCard>
            </div>
        </div>
    </section>
</template>

<script>
  import ociLogo from '../../assets/images/oci-logo.png'
  import mavenLogo from '../../assets/images/maven-logo.png'
  import PackageCard from '../components/PackageCard'

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
      ociLogo: () => (ociLogo),
      mavenLogo: () => (mavenLogo)
    },
    methods: {
      async fetchContainersCount () {
        const { count } = await this.$containers.list();
        this.containersCount = count;
      }
    },
    created () {
      this.fetchContainersCount().catch((err) => this.$emit('error', err))
    }
  }
</script>

<style scoped>

</style>