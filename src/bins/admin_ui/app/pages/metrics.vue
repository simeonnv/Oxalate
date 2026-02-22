<script setup lang="ts" >

interface resActiveTasks {
    active_tasks: Record<string, ActiveTask>
}

interface ActiveTask{
    created_at: string,
    last_rellocated: string,
    dead: boolean,
    task: Task
}

interface Task {
    proxy_reqs: ProxyReq[]
}

interface ProxyReq{
    Http: HttpRequest
}

interface HttpRequest {
    url: string,
    body: string,
    headers: Record<string, string>
    method: string
}


interface resProxy{
    connected_proxies: string[]
}

const {data, pending, error, refresh} = useFetch<resActiveTasks>("http://localhost:6969/metric/active_tasks");
const {data: proxy_data, error: proxy_error, refresh: proxy_refresh} = useFetch<resProxy>("http://localhost:6969/metric/connected_proxies")


</script>


<template>

<div class="flex gap-4">

    <div class="overflow-x-auto rounded-box border border-base-content/5 bg-base-100">
    <table class="table">
        <!-- head -->
        <thead>
        <tr>
            <th>Created at</th>
            <th>Status</th>
            <th>Proxy requests</th>
            
        </tr>
        </thead>
        <tbody>
            <tr v-for="(task, id) in data?.active_tasks" :key="id">
                <td> {{ new Date(task.created_at).toLocaleString() }}</td>
                <td>
                    <div :class="task.dead ? 'badge badge-error' : 'badge badge-success'">
                        {{ task.dead ? 'Dead' : 'Active' }}
                    </div>
                </td>
                <td>{{ task.task.proxy_reqs.length }}</td>
        
         </tr>
        </tbody>
    </table>
    </div>

    <div class="overflow-x-auto rounded-box border border-base-content/5 bg-base-100">
    <div class="overflow-x-auto rounded-box border border-base-content/5 bg-base-100">
    <table class="table">
        <!-- head -->
        <thead>
        <tr>
            <th>Index</th>
            <th>Proxy id</th>
            
            
        </tr>
        </thead>
        
        
        <tbody>
            <tr v-for="(proxy_id, index) in proxy_data?.connected_proxies">
                <td>{{ index }}</td>
                <td>{{ proxy_id }}</td>
                
        
            </tr>
        </tbody>
    </table>
    </div>



    
    </div>

</div>





</template>