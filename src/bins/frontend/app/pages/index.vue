<script setup lang="ts">
const router = useRouter();
const input_text = ref("");

const handleSearch = () => {
    router.push({
        
        path: "/result",
        query: { q: input_text.value },
    });
};
</script>

<template>
    <div
        class="h-full w-full flex flex-col items-center align-middle justify-center gap-6"
    >
        <div
            class="flex flex-row justify-center items-center align-middle gap-1"
        >
            <p class="text-8xl">[</p>
            <p class="text-6xl font-bold">Oxalate</p>
            <p class="text-8xl">]</p>
        </div>

        <label class="input input-xl w-md">
            <Icon name="gravity-ui:magnifier" />
            <input
                v-model="input_text"
                class="input-xl"
                type="search"
                required
                placeholder="Search..."
                @keypress.enter="handleSearch"
            />
        </label>
    </div>
</template>

<style scoped>
div {
    font-family: "3270-Regular";
}
</style>
