<script setup lang="ts">
definePageMeta({
    layout: 'search-bar',
})

import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";

interface SearchResponse {
    search_results: SearchResult[]
    metasearch_results: SearchResult[]
}


interface SearchResult{
    url: string,
    score: number
}


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

const {data: results, pending, error} = useFetch<SearchResponse>('http://localhost:22267/search', {
    method: 'POST',
    body: computed(() => ({
        text: query.value
    })),
    watch: [query],
    immediate: !!query.value
})

if (error) {
    console.log(error);
}

watch(results, (newVal) => {
    console.log("Raw API Response:", newVal);
});


const filtered_metasearch = computed(() => {
  return results.value?.metasearch_results?.filter(item => item.score > 1) || [];
});

</script>

<template>


    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 min-h-125">

        <div class="bg-base-100 rounded-box shadow-md flex flex-col grow overflow-hidden">
                
                <div v-if="pending" class="flex grow items-center justify-center">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            
                <div v-else-if="error" class="flex grow items-center justify-center gap-2">
                    <div class="relative flex h-3 w-3">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-error opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-3 w-3 bg-error"></span>
                    </div>
                    <p>Error fetching data</p>
                </div>
            
                <div v-else-if="results?.search_results?.length" class="flex flex-col h-full">
                    <p class="p-4 pb-2 text-xl opacity-60 tracking-wide">Search results:</p>
                    <ul class="overflow-y-auto px-4 pb-4">
                        <li 
                            v-for="item in results.search_results" 
                            :key="item.url"
                            class="border-b border-base-200 py-3 flex justify-between items-center"
                        >
                            <a class="text-base truncate mr-4" :href="item.url">{{ item.url }}</a>
                            <div class="badge badge-ghost font-mono">{{ item.score }}</div>
                        </li>
                    </ul>
                </div>
                
                <div v-else class="flex grow items-center justify-center">
                    <p class="text-4xl opacity-80">Found nothing!</p>
                </div>
            </div>



            <div class="bg-base-100 rounded-box shadow-md flex flex-col grow overflow-hidden">
                
                <div v-if="pending" class="flex grow items-center justify-center">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            
                <div v-else-if="error" class="flex grow items-center justify-center gap-2">
                    <div class="relative flex h-3 w-3">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-error opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-3 w-3 bg-error"></span>
                    </div>
                    <p>Error fetching data</p>
                </div>
            
                <div v-else-if="filtered_metasearch?.length" class="flex flex-col h-full">
                    <p class="p-4 pb-2 text-xl opacity-60 tracking-wide">Search results:</p>
                    <ul class="overflow-y-auto px-4 pb-4">
                        <li 
                            v-for="item in filtered_metasearch" 
                            :key="item.url"
                            class="border-b border-base-200 py-3 flex justify-between items-center"
                        >
                            <a class="text-base truncate mr-4" :href="item.url">{{ item.url }}</a>
                            <div class="badge badge-ghost font-mono">{{ item.score }}</div>
                        </li>
                    </ul>
                </div>
                
                <div v-else class="flex grow items-center justify-center">
                    <p class="text-4xl opacity-80">Found nothing!</p>
                </div>
            </div>

    </div>

    
</template>