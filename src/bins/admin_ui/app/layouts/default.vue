<template>
  <div class="drawer lg:drawer-open">
    <input id="my-drawer-4" type="checkbox" class="drawer-toggle" />
    
    <div class="drawer-content flex flex-col">
      <nav class="navbar w-full bg-base-300">
        <label for="my-drawer-4" aria-label="open sidebar" class="btn btn-square btn-ghost">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" stroke-linejoin="round" stroke-linecap="round" stroke-width="2" fill="none" stroke="currentColor" class="my-1.5 inline-block size-4">
            <path d="M4 4m0 2a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z"></path>
            <path d="M9 4v16"></path><path d="M14 10l2 2l-2 2"></path>
          </svg>
        </label>
        <div class="flex flex-row justify-center items-center align-middle gap-1 ml-2">
          <p class="text-3xl">[</p>
          <p class="text-3xl font-bold">Oxalate</p>
          <p class="text-3xl">]</p>
        </div>
      </nav>

      <main class="p-4">
        <slot /> 
      </main>
    </div>

    <div class="drawer-side is-drawer-close:overflow-visible">
      <label for="my-drawer-4" aria-label="close sidebar" class="drawer-overlay"></label>
      <div class="flex min-h-full flex-col items-start bg-base-200 is-drawer-close:w-14 is-drawer-open:w-64">
        <ul class="menu w-full grow">
          <li>
            <NuxtLink to="/" class="flex items-center" active-class="active">
              <button class="is-drawer-close:tooltip is-drawer-close:tooltip-right flex items-center gap-2" data-tip="Homepage">
                <Icon name="carbon:home" class="text-2xl"></Icon>
                <span class="is-drawer-close:hidden">Homepage</span>
              </button>
            </NuxtLink>
          </li>
          <li>
            <NuxtLink to="/metrics" class="flex items-center" active-class="active">
              <button class="is-drawer-close:tooltip is-drawer-close:tooltip-right flex items-center gap-2" data-tip="Metrics">
                <Icon name="carbon:ai-status" class="text-2xl"></Icon>
                <span class="is-drawer-close:hidden">Metrics</span>
              </button>
            </NuxtLink>
          </li>
          <li>
            <NuxtLink to="/control" class="flex items-center" active-class="active">
              <button class="is-drawer-close:tooltip is-drawer-close:tooltip-right flex items-center gap-2" data-tip="Controls">
                <Icon name="carbon:gears" class="text-2xl"></Icon>
                <span class="is-drawer-close:hidden">Controls</span>
              </button>
            </NuxtLink>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>