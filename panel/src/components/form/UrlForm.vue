<template>
  <div
    v-if="show_delete_confirm"
    class="app-modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="delete-link-title"
  >
    <div class="app-modal confirmation-modal">
      <div class="confirmation-icon">
        <Icon :icon="faQuestion" aria-hidden="true" />
      </div>
      <h3 id="delete-link-title">Delete this link?</h3>
      <p>This action cannot be undone</p>
      <strong>{{ id }}: {{ url?.slice(0, 40) }}{{ url?.length > 40 ? '…' : '' }}</strong>
      <div class="confirmation-actions">
        <button class="app-button app-button-danger" type="button" @click="$emit('on_delete')">
          Delete link
        </button>
        <button class="app-button" type="button" @click="show_delete_confirm = false">
          Cancel
        </button>
      </div>
    </div>
  </div>

  <fieldset class="app-panel url-form-card" :disabled="show_delete_confirm">
    <legend>{{ id ? `Link details · ${id}` : 'Link details' }}</legend>

    <label class="app-field" for="destination-url">
      <span>Destination URL</span>
      <span class="app-control" :class="{ 'is-invalid': is_url_allready_use }">
        <Icon :icon="faLink" aria-hidden="true" />
        <input id="destination-url" v-model="url" type="url" placeholder="https://example.com" />
      </span>
    </label>
    <p v-if="is_url_allready_use" class="app-field-error">
      This destination is already in use
    </p>

    <label class="app-field" for="short-code">
      <span>Short code <span class="optional-label">Optional</span></span>
      <span class="app-control" :class="{ 'is-invalid': is_slug_allready_use }">
        <Icon :icon="faChain" aria-hidden="true" />
        <input id="short-code" v-model="slug" type="text" placeholder="Add a custom code" />
      </span>
    </label>
    <p v-if="is_slug_allready_use" class="app-field-error">
      This short code is already in use
    </p>

    <label class="app-field" for="link-type">
      <span>Link type</span>
      <span class="app-control">
        <Icon :icon="faSlash" aria-hidden="true" />
        <select id="link-type" v-model="url_type">
          <option disabled value="">Select a type</option>
          <option value="image">Image</option>
          <option value="video">Video</option>
          <option value="url">URL</option>
        </select>
      </span>
    </label>

    <div class="app-field">
      <span>Categories</span>
      <div class="category-picker">
        <div class="app-chip-group">
          <label
            v-for="category in categories_list"
            :key="category.id"
            class="app-filter-chip"
          >
            <input
              v-model="categories"
              :value="category.id"
              type="checkbox"
              name="url-categories"
            />
            <span>{{ category.name }}</span>
          </label>
          <span v-if="!categories_list.length" class="category-empty">
            No categories available
          </span>
        </div>
        <button
          v-if="categories.length"
          class="app-button app-button-quiet clear-categories"
          type="button"
          @click="categories = []"
        >
          Clear
        </button>
      </div>
    </div>

    <div class="form-actions">
      <button
        v-if="id"
        class="app-button app-button-danger"
        type="button"
        :disabled="!is_correct"
        @click="show_delete_confirm = true"
      >
        Delete link
      </button>
      <button
        class="app-button app-button-primary"
        type="button"
        :disabled="!is_correct"
        @click="$emit('on_save')"
      >
        Save link
      </button>
    </div>

    <section class="preview-section" aria-labelledby="link-preview-title">
      <div class="app-panel-heading preview-heading">
        <div>
          <p class="app-kicker">Destination</p>
          <h3 id="link-preview-title">Preview</h3>
        </div>
      </div>
      <div class="link-preview">
        <img
          v-if="url_type === 'image' && url"
          class="preview-image"
          :src="url"
          alt="Destination preview"
        />
        <VideoPreview
          v-else-if="url_type === 'video' && url"
          class="preview-video"
          :url="url.includes('youtu') ? provide_embed_url_for_any_yotube_urls(url) : url"
        />
        <a
          v-if="url"
          class="app-table-link preview-destination"
          target="_blank"
          rel="noopener noreferrer"
          :href="url"
        >
          {{ url }}
        </a>
        <p v-else class="preview-empty">Add a destination URL to see its preview</p>
      </div>
    </section>
  </fieldset>
</template>

<script setup>
//07aN0rFRe8Y

import VideoPreview from '@/components/VideoPreview.vue'
import { faChain, faLink, faSlash, faQuestion } from '@fortawesome/free-solid-svg-icons'
import { onMounted, ref, watch } from 'vue'
import axios from 'axios'

import { useUrlStore } from '@/stores/url_form_store';
import { storeToRefs } from "pinia";



const provide_embed_url_for_any_yotube_urls = youtube_url => {
    if (youtube_url.includes('embed')){
        return youtube_url
    } else {
        return `https://www.youtube.com/embed/${youtube_url.replace('https://www.youtube.com/watch?v=v', '').replace('https://youtu.be/', '')}`
    }
}


const url_store = useUrlStore()
const { id, url, url_type, slug, categories } = storeToRefs(url_store)
const emit = defineEmits(['on_save', 'on_delete'])


const show_delete_confirm = ref(false)
const is_correct = ref(false)
const url_list = ref([])
const categories_list = ref([])

const load_url_list = async () => {
    var r = await axios.get('/admin/list')
    url_list.value = r.data
}

const load_categories_list = async () => {
    var r = await axios.get('/admin/category/')
    categories_list.value = r.data
}

const is_url_allready_use = ref(false)
const is_slug_allready_use = ref(false)

watch([url, url_type, slug, url_list], ([newUrl, , newSlug]) => {
  const hasValidUrl = typeof newUrl === 'string' &&
    newUrl.startsWith('http') &&
    newUrl.length > 0
  const urlAlreadyUsed = url_list.value.some(
    item => item.url === newUrl && item.id !== id.value
  )
  const slugAlreadyUsed = Boolean(newSlug) && url_list.value.some(
    item => item.code === newSlug && item.id !== id.value
  )

  is_url_allready_use.value = urlAlreadyUsed
  is_slug_allready_use.value = slugAlreadyUsed
  is_correct.value = hasValidUrl && !urlAlreadyUsed && !slugAlreadyUsed
})

onMounted(load_url_list)
onMounted(load_categories_list)

</script>

<style scoped>
.url-form-card {
  display: grid;
  gap: 16px;
  min-inline-size: 0;
}

.url-form-card legend {
  padding: 0 8px;
  color: var(--app-text-secondary);
  font-size: 13px;
}

.url-form-card .app-control.is-invalid {
  border-color: var(--app-danger);
}

.optional-label {
  margin-left: 4px;
  color: var(--app-text-muted);
  font-size: 12px;
}

.category-picker {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.category-empty,
.preview-empty {
  margin: 0;
  color: var(--app-text-muted);
  font-size: 13px;
}

.clear-categories {
  min-height: 32px;
  padding: 0 8px;
  font-size: 12px;
}

.form-actions,
.confirmation-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.form-actions .app-button-danger {
  margin-right: auto;
}

.preview-section {
  display: grid;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid var(--app-border);
}

.preview-heading {
  margin-bottom: 0;
}

.preview-heading .app-kicker {
  margin-bottom: 2px;
}

.preview-heading h3 {
  font-size: 16px;
}

.link-preview {
  display: grid;
  gap: 12px;
  min-width: 0;
}

.preview-image {
  width: 100%;
  max-height: 320px;
  border: 1px solid var(--app-border);
  border-radius: 10px;
  background: var(--app-raised);
  object-fit: contain;
}

.preview-video {
  width: 100%;
  aspect-ratio: 16 / 9;
  height: auto;
  border: 0;
  border-radius: 10px;
}

.preview-destination {
  overflow-wrap: anywhere;
}

.confirmation-modal h3 {
  margin: 12px 0 4px;
  font-size: 20px;
  font-weight: 500;
}

.confirmation-modal p {
  margin: 0 0 12px;
  color: var(--app-text-secondary);
}

.confirmation-modal strong {
  display: block;
  overflow-wrap: anywhere;
  color: var(--app-text);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 13px;
}

.confirmation-icon {
  display: grid;
  width: 36px;
  height: 36px;
  place-items: center;
  border-radius: 9px;
  background: var(--app-raised);
  color: var(--app-text-secondary);
}

.confirmation-actions {
  justify-content: flex-end;
  margin-top: 24px;
}

@media (max-width: 480px) {
  .category-picker {
    flex-direction: column;
  }

  .clear-categories {
    align-self: flex-end;
  }

  .form-actions {
    flex-direction: column-reverse;
    align-items: stretch;
  }

  .confirmation-actions {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>