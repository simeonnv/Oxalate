<script setup lang="ts">
let input_text = ref("");

</script>

<template>
    <div class="h-full w-full flex flex-col items-center align-middle justify-center gap-6">
        <div class="flex flex-row justify-center items-center align-middle gap-1">
          <p class="text-8xl">[</p>
          <p class="text-6xl font-bold">Oxalate</p>
          <p class="text-8xl">]</p>
        </div>
        
        <label class="input input-xl w-md">
          <Icon name="gravity-ui:magnifier" />
          <input 
            :value="input_text" 
            @input="event => {
              const target = event.target as HTMLInputElement;
              input_text = target.value;
            }" 
            class="input-xl" type="search" required placeholder="Search" />
        </label>
    </div>
</template>

<style scoped>
div {
  font-family: "3270-Regular";
}
</style>