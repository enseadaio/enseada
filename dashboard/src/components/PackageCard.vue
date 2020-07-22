<template>
    <router-link :to="to">
        <div class="box has-custom-background" :style="bgColorStyle">
            <article class="media">
                <div class="media-left">
                    <figure class="image is-64x64 has-background-white">
                        <img :src="image" />
                    </figure>
                </div>
                <div class="media-content">
                    <div class="">
                        <h2 class="title has-text-white">{{ title }}</h2>
                        <h3 class="subtitle has-text-white">
                            <slot></slot>
                        </h3>
                    </div>
                </div>
            </article>
        </div>
    </router-link>
</template>

<script>
  export default {
    name: 'PackageCard',
    props: {
      to: String | Object,
      bgColor: {
        type: String,
        default: '#FFFFFF'
      },
      image: String,
      title: String,
    },
    computed: {
      bgColorStyle() {
        return `background-color: ${this.bgColor}`
      }
    },
  }
</script>

<style scoped>
</style>