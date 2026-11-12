<template>
  <main class="app-page">
    <header class="app-page-heading">
      <div>
        <p class="app-kicker">URL management</p>
        <h2>Your links</h2>
        <p class="app-description">Create, organize, and review your tracked links</p>
      </div>
      <div class="app-actions">
        <RouterLink class="app-button" :to="{ name: 'category-manage' }">
          Manage categories
        </RouterLink>
        <RouterLink class="app-button app-button-primary" :to="{ name: 'add-url' }">
          New link
        </RouterLink>
      </div>
    </header>

    <section class="app-panel links-panel" aria-label="Short link list">
      <div class="links-toolbar">
        <div class="category-filter">
          <div class="filter-heading">
            <span>Categories</span>
            <button
              class="app-button app-button-quiet clear-filter"
              type="button"
              :disabled="!selected_categories_ids.length"
              @click="selected_categories_ids = []"
            >
              Clear
            </button>
          </div>
          <div class="app-chip-group">
            <label
              v-for="category in categories_list"
              :key="category.id"
              class="app-filter-chip"
            >
              <input
                v-model="selected_categories_ids"
                :value="category.id"
                type="checkbox"
                name="categories"
              />
              <span>{{ category.name }}</span>
            </label>
            <span v-if="!categories_list.length" class="filter-empty">
              No categories
            </span>
          </div>
        </div>

        <label class="app-control link-search" :class="{ 'is-invalid': query_form_is_vaild }">
          <Icon :icon="faSearch" aria-hidden="true" />
          <input
            v-model="query"
            type="search"
            aria-label="Search links or enter a URL to create"
            placeholder="Search links or paste a URL to create"
            :aria-invalid="query_form_is_vaild"
            @keydown.enter.prevent="addNewUrl"
          />
        </label>
        <p v-if="query_form_is_vaild" class="app-field-error search-error">
          Enter a valid URL that is not already in your links
        </p>
      </div>

      <div class="app-table-wrap">
        <table class="app-table links-table">
          <thead>
            <tr>
              <th scope="col" class="row-number">#</th>
              <th scope="col" class="destination-column">Destination</th>
              <th scope="col">Short code</th>
              <th scope="col" class="number-cell">Visits</th>
              <th scope="col">Type</th>
              <th scope="col">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(url, index) in result_list" :key="url.code">
              <th scope="row" class="row-number">{{ index + 1 }}</th>
              <td class="destination-column">
                <a
                  :href="url.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="app-table-link"
                >
                  {{ url.url?.substring(0, 80) }}
                  {{ url.url?.length > 80 ? '…' : '' }}
                </a>
              </td>
              <td>
                <button
                  class="code-copy app-code"
                  type="button"
                  :aria-label="copiedCode === url.code ? 'Short link copied' : 'Copy short link'"
                  @click="copyCode(url.code)"
                >
                  {{ copiedCode === url.code ? 'Copied' : url.code }}
                </button>
              </td>
              <td class="number-cell">
                <RouterLink
                  class="visits-link"
                  :to="{
                    name: 'analytical-view',
                    params: { url_id: url.id, url_code: url.code }
                  }"
                  :aria-label="`View ${url.click_count} visits`"
                >
                  {{ url.click_count }}
                  <Icon :icon="faEye" aria-hidden="true" />
                </RouterLink>
              </td>
              <td>
                <span class="type-cell" :title="url.preview_type">
                  <Icon
                    v-if="url.preview_type === 'image'"
                    :icon="faImage"
                    aria-hidden="true"
                  />
                  <Icon
                    v-else-if="url.preview_type === 'video'"
                    :icon="faVideo"
                    aria-hidden="true"
                  />
                  <Icon
                    v-else-if="url.preview_type === 'url'"
                    :icon="faLink"
                    aria-hidden="true"
                  />
                  {{ url.preview_type || '—' }}
                </span>
              </td>
              <td>
                <RouterLink
                  class="app-button app-button-quiet row-action"
                  :to="{ name: 'manage-url', params: { url_code: url.code } }"
                >
                  Edit
                </RouterLink>
              </td>
            </tr>
            <tr v-if="!result_list.length">
              <td colspan="6" class="app-empty-state">
                {{ url_list.length ? 'No links match these filters' : 'No links yet' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </main>
</template>

<script setup>
import axios from 'axios'
import { ref, onMounted, computed, watch } from 'vue'
import { useClipboard } from '@vueuse/core'
import { RouterLink } from 'vue-router'
import { faSearch, faImage, faVideo, faLink, faEye } from '@fortawesome/free-solid-svg-icons'
import { useUrlStore } from '@/stores/url_form_store';
import { storeToRefs } from "pinia";

const url_store = useUrlStore()
const { url } = storeToRefs(url_store)
const url_base = import.meta.env.VITE_API_URL

const url_list = ref([])
const categories_list = ref([])
const selected_categories_ids = ref([])

const query = ref('')
const query_form_is_vaild = ref(false)

const copiedCode = ref(null)
const { copy } = useClipboard()

const hasIntersection = (a, b) => {
  const setB = new Set(b)
  return a.some(x => setB.has(x))
}

const result_list = computed(() => {
  const q = query.value.toLowerCase().trim()
  const selected = selected_categories_ids.value

  return url_list.value.filter(item => {
    const text = `${item.url} ${item.code}`.toLowerCase()

    const textMatch = !q || text.includes(q)
    const categoryMatch =
      selected.length === 0 ||
      hasIntersection(selected, item.categories)

    return textMatch && categoryMatch
  })
})

watch(query, ()=>{query_form_is_vaild.value=false})

const addNewUrl = async () => {
  if (!query.value.startsWith('http')) {
    query_form_is_vaild.value = true
    return
  }

  const exists = url_list.value.some(e => e.url === query.value)

  if (exists) {
    query_form_is_vaild.value = true
    return
  }

  query_form_is_vaild.value = false
  url.value = query.value

  await url_store.create_new_url()
  url_store.flush_url_data()
  await load_url_list()
}

const copyCode = (code) => {
  copy(`${url_base}/r/${code}`)
  copiedCode.value = code
  setTimeout(() => (copiedCode.value = null), 1200)
}

const load_categories_list = async () => {
  const r = await axios.get('/admin/category/')
  categories_list.value = r.data
}

const load_url_list = async () => {
  const r = await axios.get('/admin/list')
  url_list.value = r.data
}

onMounted(async () => {
  await load_url_list()
  load_categories_list()
})
</script>

<style scoped>
.links-panel {
  padding: 0;
  overflow: hidden;
}

.links-toolbar {
  display: grid;
  gap: 16px;
  padding: 20px 24px;
  border-bottom: 1px solid var(--app-border);
}

.category-filter {
  display: grid;
  gap: 8px;
}

.filter-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--app-text-secondary);
  font-size: 13px;
}

.clear-filter {
  min-height: 28px;
  padding: 0 8px;
  font-size: 12px;
}

.filter-empty {
  color: var(--app-text-muted);
  font-size: 13px;
}

.link-search {
  width: 100%;
}

.link-search.is-invalid {
  border-color: var(--app-danger);
}

.search-error {
  margin: -8px 0 0;
}

.links-table .destination-column {
  width: 48%;
}

.row-number {
  width: 44px;
  color: var(--app-text-muted);
  font-variant-numeric: tabular-nums;
}

.number-cell {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.code-copy {
  padding: 4px 0;
  border: 0;
  background: transparent;
  color: var(--app-text-secondary);
  cursor: pointer;
  font-size: 13px;
}

.code-copy:hover {
  color: var(--app-accent);
}

.visits-link,
.type-cell {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.visits-link {
  color: var(--app-text-secondary);
  text-decoration: none;
}

.visits-link:hover {
  color: var(--app-accent);
}

.type-cell {
  color: var(--app-text-secondary);
  text-transform: capitalize;
}

.row-action {
  min-height: 32px;
  padding: 0 8px;
  font-size: 12px;
}

@media (max-width: 720px) {
  .links-table .destination-column {
    min-width: 180px;
  }
}

@media (max-width: 480px) {
  .links-toolbar {
    padding: 16px;
  }

  .links-panel {
    padding: 0;
  }

  .links-table th,
  .links-table td {
    padding: 10px 6px;
  }

  .links-table th:first-child,
  .links-table td:first-child {
    display: none;
  }
}
</style>
