<script setup lang="ts">
definePageMeta({
    layout: 'search-bar',
})

import { ref, watch, computed } from "vue";
import { useRoute, useRouter } from "vue-router";

interface SearchResult {
    url: string;
    text: string;
    title: string;
}

interface SearchResponse {
    search_results: Record<string, SearchResult[]>;
}

interface FusedResult extends SearchResult {
    frequency: number;
    engines: string[];
}

const route = useRoute();
const router = useRouter();
const query = computed(() => (route.query.q as string) ?? '');

let search_text = ref<string>(query.value);

const handleSearch = () => {
    router.push({        
        path: "/search",
        query: { q: search_text.value },
    });
};

const { data: results, pending, error } = useFetch<SearchResponse>('http://localhost:22267/search', {
    method: 'POST',
    body: computed(() => ({
        text: query.value
    })),
    watch: [query],
    immediate: !!query.value
});

if (error) {
    console.log(error);
}

watch(results, (newVal) => {
    console.log("Raw API Response:", newVal);
});

const fusedResults = computed<FusedResult[]>(() => {
    if (!results.value?.search_results) return [];

    const map = new Map<string, FusedResult>();

    for (const [engine, items] of Object.entries(results.value.search_results)) {
        for (const item of items) {
            const key = item.url;
            
            if (map.has(key)) {
                const existing = map.get(key)!;
                existing.frequency += 1;
                existing.engines.push(engine);
            } else {
                map.set(key, {
                    ...item,
                    frequency: 1,
                    engines: [engine]
                });
            }
        }
    }

    return Array.from(map.values()).sort((a, b) => b.frequency - a.frequency);
});

const getEngineColor = (engine: string) => {
    const colors = ['bg-primary', 'bg-secondary', 'bg-accent', 'bg-info', 'bg-success', 'bg-warning'];
    let hash = 0;
    for (let i = 0; i < engine.length; i++) {
        hash = engine.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length] + ' text-primary-content';
};

</script>

<template>
    <div class="p-4 min-h-125 max-w-5xl mx-auto">
        <div class="bg-base-100 border-2 rounded-none border-base-200 flex flex-col grow overflow-hidden p-4">
                
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
        
            <div v-else-if="fusedResults.length" class="flex flex-col h-full">
                <p class="text-xl px-2 py-2 opacity-60 tracking-wide">Combined Search Results:</p>
                <ul class="overflow-y-auto flex flex-col gap-4">
                    <li 
                        v-for="item in fusedResults" 
                        :key="item.url"
                        class="border-2 p-4 border-base-200 py-4 flex flex-col"
                    >
                        <div class="flex justify-between items-start gap-4">
                            <a class="text-lg font-semibold text-primary truncate" :href="item.url">
                                {{ item.title || item.url }}
                            </a>
                            
                            <div class="flex flex-wrap gap-1 shrink-0">
                                <div 
                                    v-for="engine in item.engines" 
                                    :key="engine"
                                    class="flex items-center justify-center p-2 w-fit h-6 text-xs font-bold rounded-none cursor-help"
                                    :class="getEngineColor(engine)"
                                    :title="engine"
                                >
                                    {{ engine }}
                                </div>
                            </div>
                            
                        </div>
                        <a class="text-xs text-base-content/70 truncate" :href="item.url">{{ item.url }}</a>
                        <p class="text-sm line-clamp-2 text-base-content/80">{{ item.text }}</p>
                    </li>
                </ul>
            </div>
            
            <div v-else class="flex grow items-center justify-center">
                <p class="text-4xl opacity-80">Found nothing!</p>
            </div>
        </div>
    </div>
</template>