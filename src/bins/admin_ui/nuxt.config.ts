/**
 * Copyright 2026 Simeon Vangelov
 * SPDX-License-Identifier: GPL-3.0-only
 */

import tailwindcss from "@tailwindcss/vite";
// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  vite: {
    plugins: [tailwindcss() as any],
  },
  css: ["./app/tailwind.css"],
  modules: ['@nuxt/icon', '@nuxt/hints', '@nuxt/eslint']
})