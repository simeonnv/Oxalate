<script lang="ts">


declare module 'nuxt/app' {
  interface NuxtLayouts {
    'default': unknown
  }
}
// ---cut---
definePageMeta({
  layout: 'default',
})

</script>



<template>
  
    <NuxtLayout>
      <RouterView/>
    </NuxtLayout>
  
</template>
