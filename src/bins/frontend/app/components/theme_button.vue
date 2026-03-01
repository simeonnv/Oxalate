<script lang="ts" setup>
const colorMode = useColorMode()

const themes = [
  'light', 'dark', 'synthwave', 'retro', 'cyberpunk',
  'halloween', 'forest', 'aqua', 'black', 'luxury',
  'dracula', 'business', 'night', 'coffee', 'dim',
  'sunset', 'abyss'
]
</script>

<template>
  <div class="dropdown dropdown-end">
    <div tabindex="0" role="button" class="btn btn-outline btn-square m-1 rounded-none border border-base-300">
       <Icon name="mdi:format-paint"/>
    </div>
    <ul 
      tabindex="0" 
      class="dropdown-content menu bg-base-100 p-2 shadow-sm overflow-y-auto max-h-80 w-52 rounded-none border border-base-300 z-[1]"
    >
      <li v-for="theme in themes" :key="theme">
        <button 
          @click="colorMode.preference = theme"
          :class="{ 'active': colorMode.preference === theme }"
          class="rounded-none"
        >
          <span class="capitalize">{{ theme }}</span>
          <Icon v-if="colorMode.preference === theme" name="mdi:check" class="ml-auto" />
        </button>
      </li>
    </ul>
  </div>
</template>