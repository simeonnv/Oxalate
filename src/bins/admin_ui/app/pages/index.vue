<script setup lang="ts" >

interface resScraperState{
    enabled: boolean
}


const { data, status, error, refresh, clear } = await useFetch<resScraperState>('http://localhost:6969/control/scraper_state')


</script>


<template>
    <div v-if="data?.enabled === true" class=" items-center justify-middle align-middle flex flex-row gap-2">
        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-success animate-ping"></div>
            <div class="status status-success"></div>
        </div>
        <p>Server is up</p>
        
    </div>
    <div v-else-if="data?.enabled === false" class=" items-center justify-middle flex flex-row ">

        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-error animate-ping"></div>
            <div class="status status-error"></div>
        </div>
        <p>Server is down</p>

        
    </div> 
    <div v-else-if="error" class=" items-center flex flex-row ">

        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-error animate-ping"></div>
            <div class="status status-error"></div>
        </div>
        <p>Error fetching</p>

    </div>
    


</template>