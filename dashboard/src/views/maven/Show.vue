<template>
  <b-loading :is-full-page="true" v-if="loading" v-model="loading"></b-loading>
  <section class="section" v-else>
    <h1 class="title">{{ model.group_id }}:{{ model.artifact_id }}</h1>
    <TreeBrowser
        :node="tree"
        @onClick="alert(JSON.stringify($event))"
    />
  </section>
</template>

<script>
import TreeBrowser from '../../components/TreeBrowser'
import showPage from '../../mixins/showPage'
import { buildFileTree } from '../../utils'

export default {
  name: 'MavenShow',
  components: { TreeBrowser },
  mixins: [showPage({
    name: 'repository',
    service: 'maven',
    permission: { object: 'maven_repos', action: 'read' },
    idProps: {
      group_id: String,
      artifact_id: String
    },
    mapId: ({ group_id, artifact_id }) => `${group_id}/${artifact_id}`
  })
  ],
  data () {
    return {
      model: null,
      files: []
    }
  },
  computed: {
    tree () {
      return buildFileTree(this.files)
    }
  },
  methods: {
    async fetchFiles () {
      const fileService = this.$maven.association('files', this.mapId(this.$props))
      this.files = await fileService.list()
      console.log(this.files)
    }
  },
  created () {
    this.fetchFiles().catch((err) => this.$emit('error', err))
  }
}
</script>

<style scoped>

</style>