<script lang="ts" setup>
const route = useRoute()
const router = useRouter();
const query = computed(() => (route.query.q as string) ?? '');

let search_text = ref<string>(query.value);
const handleSearch = () => {
    router.push({        
        path: "/search",
        query: { q: search_text.value },
    });
};
</script>

<template>
    <div class="flex flex-col h-screen box-border">
        
        <div class="navbar bg-base-200 shadow-sm flex flex-row rounded-xl shrink-0">
            <NuxtLink to="/">
                <div class="flex hover:bg-base-300 ease-in-out transition-all rounded-xl flex-row justify-center items-center gap-1 m-2 p-2">
                    <span class="text-6xl">[</span>
                    <span class="text-4xl font-bold">Oxalate</span>
                    <span class="text-6xl">]</span>
                </div>
            </NuxtLink>
            <div class="grow px-4">
                <input 
                    v-model="search_text"
                    @keypress.enter="handleSearch"
                    type="text" 
                    placeholder="Search" 
                    class="input input-bordered w-full" 
                />
            </div>
        </div>

        <slot/>
    </div>
</template>
    