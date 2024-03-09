<template>
  <div>
    <nuxt-page/>
  </div>
</template>

<script setup>
useHead({
  script: [
    {
      src: '/w/index.js',
      defer: true,
    },
  ],
})
</script>
