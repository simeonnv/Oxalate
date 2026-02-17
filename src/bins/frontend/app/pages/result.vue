<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRoute } from "vue-router";

interface SearchResponse {
    search_results: SearchResult[]
}


interface SearchResult{
    url: string,
    score: number
}




const route = useRoute()
const query = computed(() => (route.query.q as string) ?? '');


const {data: results, pending, error} = await useFetch<SearchResponse>('http://localhost:22267/search', {
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

</script>


<template>
    


   
   
    <ul class="list bg-base-100 rounded-box shadow-md">
      
        <div v-if="pending">
            <span class="loading loading-spinner loading-xl"></span>
        </div>
    
        <div v-else-if="error">
            <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-error animate-ping"></div>
            <div class="status status-error"></div>
            </div> Error fetching data
    
        </div>
    
    
    
    
        <div class="flex flex-col gap-2" v-else-if="results?.search_results.length">
            <li class="p-4 pb-2 text-xl opacity-60 tracking-wide">Search results:</li>
        
            <li class =" border-3 mx-50 list-row ;" v-for="item in results.search_results">
                <div class="text-base">{{ item.url }}</div>
                <div class="text-xs uppercase font-semibold opacity-60">{{ item.score }}</div>
            </li>
            
        </div>
      
    </ul>










</template>