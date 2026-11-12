<template>
  <main class="app-page app-page--medium">
    <header class="app-page-heading">
      <div>
        <p class="app-kicker">Organization</p>
        <h2>Manage categories</h2>
        <p class="app-description">Group links so they are easier to find and review</p>
      </div>
    </header>

    <section class="app-panel category-panel" aria-label="Manage categories">
      <label class="app-control category-search">
        <Icon :icon="faSearch" aria-hidden="true" />
        <input
          v-model="query"
          type="search"
          aria-label="Search or add a category"
          placeholder="Search categories or enter a name to add"
          @keydown.enter.prevent="addCategory"
        />
      </label>

      <div class="app-table-wrap">
        <table class="app-table category-table">
          <thead>
            <tr>
              <th scope="col" class="row-number">#</th>
              <th scope="col">Category name</th>
              <th scope="col" class="category-action-heading">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(category, index) in filteredCategories" :key="category.id">
              <th scope="row" class="row-number">{{ index + 1 }}</th>
              <td>
                <span
                  class="category-name"
                  contenteditable
                  role="textbox"
                  aria-label="Edit category name"
                  @keydown.enter.prevent="$event.target.blur()"
                  @blur="updateCategory($event, category)"
                >
                  {{ category.name }}
                </span>
              </td>
              <td class="category-action">
                <button
                  class="app-button app-button-quiet delete-category"
                  type="button"
                  @click="deleteCategory(category.id)"
                >
                  Delete
                </button>
              </td>
            </tr>
            <tr v-if="!filteredCategories.length">
              <td colspan="3" class="app-empty-state">
                {{ categories_list.length ? 'No categories match this search' : 'No categories yet' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="category-hint">Press Enter to add a category or save a name change</p>
    </section>
  </main>
</template>

<script setup>
import axios from "axios";
import { ref, computed, onMounted } from "vue";
import { faSearch } from "@fortawesome/free-solid-svg-icons";
import slugify from "slugify";

const query = ref("");
const categories_list = ref([]);

const filteredCategories = computed(() => {
  if (!query.value) return categories_list.value;
  return categories_list.value.filter(c =>
    c.name.toLowerCase().includes(query.value.toLowerCase())
  )
})


const loadCategories = async () => {
  const r = await axios.get("/admin/category/");
  categories_list.value = r.data;
};

const addCategory = async () => {
  if (!query.value.trim()) return;

  await axios.post("/admin/category/", null, {
    params: {
      name: query.value,
      slug: slugify(query.value, { lower: true })
    }
  })

  query.value = "";

  await loadCategories()
}

const updateCategory = async (e, category) => {
    const newName = e.target.innerText.trim();
    if (newName === category.name) return;

    axios.put(`/admin/category/${category.id}`, null, {
        params: {
            name: newName,
            slug: slugify(newName, { lower: true })
        }
    })
    category.name = newName;
}


const deleteCategory = async id => {
    await axios.delete(`/admin/category/${id}`, {
        params: { id }
    })
    await loadCategories()
}

onMounted(loadCategories);
</script>

<style scoped>
.category-panel {
  padding: 0;
  overflow: hidden;
}

.category-search {
  margin: 20px 24px;
}

.category-action-heading,
.category-action {
  width: 120px;
  text-align: right;
}

.category-name {
  display: inline-block;
  min-width: min(100%, 360px);
  padding: 5px 8px;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--app-text);
}

.category-name:hover,
.category-name:focus {
  border-color: var(--app-border);
  background: var(--app-raised);
  outline: none;
}

.delete-category {
  min-height: 32px;
  padding: 0 8px;
  color: var(--app-danger);
  font-size: 12px;
}

.category-hint {
  margin: 0;
  padding: 12px 24px 16px;
  color: var(--app-text-muted);
  font-size: 12px;
}

@media (max-width: 480px) {
  .category-search {
    margin: 16px;
  }

  .category-hint {
    padding: 12px 16px 16px;
  }

  .category-action-heading,
  .category-action {
    width: 76px;
  }
}
</style>

