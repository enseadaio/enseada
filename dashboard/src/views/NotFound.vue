<template>
  <section class="section">
    <h1 class="title">Not Found</h1>
    <p v-for="reason of reasons" :key="reason">{{ reason }}</p>
  </section>
</template>

<script>
export default {
  name: 'NotFound',
  props: {
    reasons: {
      type: Array,
      default: []
    }
  }
}
</script>

<style scoped>

</style>