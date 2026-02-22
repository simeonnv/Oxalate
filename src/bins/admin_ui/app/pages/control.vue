<script setup lang="ts">


interface resScraperState{
    enabled: boolean
}


const { data, status, pending, error, refresh, clear } = useFetch<resScraperState>('http://localhost:6969/control/scraper_state')


async function swap_screaper_on_state() {
    await $fetch("http://localhost:6969/control/swap_scraper_on_state", {
        method: 'POST',
    })
}
</script>


<template>


<div v-if="data?.enabled === true" class=" items-center justify-center flex flex-row gap-2 text-4xl" >
        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-success animate-ping"></div>
            <div class="status status-success"></div>
        </div>
        <p>Server is up</p>
        
    </div>
    <div v-else-if="data?.enabled === false" class=" items-center justify-center flex flex-row gap-2 text-4xl ">

        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-error animate-ping"></div>
            <div class="status status-error"></div>
        </div>
        <p>Server is down</p>

        
    </div> 
    <div v-else-if="error" class=" items-center justify-center flex flex-row gap-2 text-4xl ">

        <div class="inline-grid *:[grid-area:1/1]">
            <div class="status status-error animate-ping"></div>
            <div class="status status-error"></div>
        </div>
        <p>Error fetching</p>

    </div>
    <div v-else-if="pending" class=" items-center justify-center flex flex-row gap-2 text-4xl ">

        <span class="loading loading-bars loading-xl"></span>
        

    </div>
    <button class="btn btn-xs sm:btn-sm md:btn-md lg:btn-lg xl:btn-xl mx-auto block mt-5 " @click="() => refresh()">Refresh</button>

    <button class="btn btn-xs sm:btn-sm md:btn-md lg:btn-lg xl:btn-xl mx-auto block mt-5 " @click="swap_screaper_on_state">
        Pause system
    </button>
    
</template>