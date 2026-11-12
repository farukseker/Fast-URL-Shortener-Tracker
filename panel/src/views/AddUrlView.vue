<template>
  <main class="app-page app-page--narrow">
    <header class="app-page-heading">
      <div>
        <p class="app-kicker">Short links</p>
        <h2>Create a short link</h2>
        <p class="app-description">Add a destination, choose its type, and organize it with categories</p>
      </div>
    </header>
    <UrlForm @on_save="save" />
  </main>
</template> 


<script setup>
import UrlForm from '@/components/form/UrlForm.vue'
import { useRouter } from "vue-router";
import { useUrlStore } from '@/stores/url_form_store';
import { onMounted } from 'vue';
import { useNotification } from '@kyvg/vue3-notification';

const { notify } = useNotification()

const url_store = useUrlStore()
const router = useRouter()


onMounted(url_store.flush_url_data)

const save = async () => {
    var r = await url_store.create_new_url()
    notify({
        title: 'Success',
        text: 'New url saved successfully',
        type: 'success',
    })
    router.push({
        name:'manage-url',
        params: {
            url_code: r.code,
        }
    })
}
</script>