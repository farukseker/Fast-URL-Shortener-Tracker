<template>
  <main class="app-page app-page--narrow">
    <header class="app-page-heading">
      <div>
        <p class="app-kicker">Short links</p>
        <h2>Edit short link</h2>
        <p class="app-description">Update the destination, short code, or categories</p>
      </div>
    </header>
    <UrlForm @on_save="on_save" @on_delete="on_delete" />
  </main>

</template>

<script setup>
import { onMounted, ref } from 'vue';
import { useUrlStore } from '@/stores/url_form_store';
import { storeToRefs } from "pinia";
import UrlForm from '@/components/form/UrlForm.vue'
import { useRouter } from 'vue-router';
import { useNotification } from '@kyvg/vue3-notification';

const { notify } = useNotification()

const { url_code } = defineProps(['url_code'])
const router = useRouter()
const url_store = useUrlStore()

const { id, url, url_type, slug, categories, pre_slug } = storeToRefs(url_store)

const load_url_data = async () => {
    url_store.load_url_data(await url_store.get_url(url_code))
}

const on_save = async () => {
    notify({
        title: 'Success',
        text: 'Url saved successfully',
        type: 'success',
    })
    await url_store.update_url()
    router.push({
        name: 'manage-url',
        params: {
            url_code: slug.value
        }
    })
} 
const on_delete = async () => {
    notify({
        title: 'Success',
        text: 'Url deleted with successfully',
        type: 'success',
    })
    await url_store.delete_url(url_code)
    router.push('/')
} 

onMounted(load_url_data)
</script>