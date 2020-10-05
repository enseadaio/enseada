<template>
  <div>
    <div
        :style="{'margin-left': `${depth * 20}px`}"
        @click="nodeClicked"
        class="node">
            <span
                class="type"
                v-if="hasChildren">{{ expanded ? '&#9660' : '&#9658' }}</span>
      <span
          v-else>&#9671;</span>
      <span
          :style="getStyle(node)">{{ node.name }}</span>
    </div>
    <TreeBrowser
        :depth="depth + 1"
        :key="child.name"
        :node="child"
        @onClick="(node) => $emit('onClick', node)"
        v-for="child in node.children"
        v-if="expanded"
    />
  </div>
</template>

<script>
export default {
  name: 'TreeBrowser',
  props: {
    node: Object,
    depth: {
      type: Number,
      default: 0
    }
  },
  data () {
    return {
      expanded: false
    }
  },
  methods: {
    nodeClicked () {
      this.expanded = !this.expanded
      if (!this.hasChildren) {
        this.$emit('click', this.node)
      }
    },
    getStyle (node) {
      let color = 'red'
      // if (!node.children) {
      // }
      return {
        color
      }
    }
  },
  computed: {
    hasChildren () {
      return this.node.children
    }
  }
}
</script>

<style scoped>
.node {
  text-align: left;
  font-size: 18px;
}

.type {
  margin-right: 5px;
}
</style>