<script lang="ts" setup>
const route = useRoute();
const router = useRouter();
const query = computed(() => (route.query.q as string) ?? '');

let search_text = ref<string>(query.value);

const handleSearch = () => {
    router.push({        
        path: route.path === '/' ? '/search' : route.path,
        query: { q: search_text.value },
    });
};

const navigateToTab = (path: string) => {
    router.push({ path, query: { q: search_text.value } });
};
</script>

<template>
    <div class="flex flex-col h-screen">
        <div class="navbar border-base-200 border-b-2 flex flex-row rounded-none shrink-0 gap-2">
            <NuxtLink to="/">
                <div class="flex hover:bg-base-300 ease-in-out transition-all p-2 rounded-none flex-row justify-center items-center gap-1">
                    <span class="text-6xl">[</span>
                    <span class="text-4xl font-bold">Oxalate</span>
                    <span class="text-6xl">]</span>
                </div>
            </NuxtLink>
            <div class="flex flex-col w-full gap-2">
                <input 
                    v-model="search_text"
                    @keypress.enter="handleSearch"
                    type="text" 
                    placeholder="Search" 
                    class="input input-bordered w-full rounded-none" 
                />
                <div class="flex flex-row gap-4">
                    <button 
                        @click="navigateToTab('/search')"
                        class="btn btn-primary btn-outline btn-xs border rounded-none"
                        :class="{ 'btn-active': route.path === '/search' }"
                    >
                        All
                    </button>

                    <!-- <button 
                        @click="navigateToTab('/images')"
                        class="btn btn-primary btn-outline btn-xs border rounded-none"
                        :class="{ 'btn-active': route.path === '/images' }"
                    >
                        Images
                    </button> -->

                    <button 
                        @click="navigateToTab('/graph')"
                        class="btn btn-primary btn-outline btn-xs border rounded-none"
                        :class="{ 'btn-active': route.path === '/graph' }"
                    >
                        Graph
                    </button>
                </div>
            </div>
            <ThemeButton/>
        </div>
        <slot/>
    </div>
</template>