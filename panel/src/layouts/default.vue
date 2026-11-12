<template>
  <div class="app-shell">
    <header class="app-header">
      <div class="app-header-inner">
        <div class="app-header-brand">
          <button
            v-if="$route.name !== 'home'"
            class="app-icon-button"
            type="button"
            aria-label="Go back"
            @click="$router.back"
          >
            <Icon :icon="faChevronLeft" aria-hidden="true" />
          </button>
          <RouterLink class="app-brand" to="/">Ulrich</RouterLink>
        </div>

        <h1 class="app-route-title">{{ $route.meta.title }}</h1>

        <div class="app-header-actions">
          <details>
            <summary class="app-icon-button" aria-label="Display settings">
              <Icon :icon="faEllipsis" aria-hidden="true" />
            </summary>
            <ul class="app-menu">
              <li>
                <button
                  type="button"
                  @click="themeStore.set_theme(themeStore.theme === 'dark' ? 'light' : 'dark')"
                >
                  Switch to {{ themeStore.theme === 'dark' ? 'light' : 'dark' }} theme
                </button>
              </li>
            </ul>
          </details>
        </div>
      </div>
    </header>

    <div class="app-main">
      <Notifications position="bottom right" :max="5" />
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import { faEllipsis, faChevronLeft } from '@fortawesome/free-solid-svg-icons'
import { useThemeStore } from '@/stores/theme'
import { Notifications } from '@kyvg/vue3-notification'

const themeStore = useThemeStore()

onMounted(() => {
  themeStore.sync_theme()
})
</script>
