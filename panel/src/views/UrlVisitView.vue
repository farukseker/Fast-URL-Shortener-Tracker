<template>
  <main class="visit-dashboard">
    <div class="app-page">
      <header class="app-page-heading">
        <div>
          <p class="app-kicker">Short link</p>
          <h2>Link activity</h2>
          <p class="app-description">Track visits over time and review where they came from</p>
        </div>
        <RouterLink
          class="app-button app-button-primary edit-link"
          :to="{ name: 'manage-url', params: { url_code: url_data.code } }"
        >
          Edit link
        </RouterLink>
      </header>

      <section class="app-panel dashboard-panel link-panel" aria-labelledby="link-details-title">
        <header class="app-panel-heading panel-heading">
          <div>
            <p class="app-kicker">Destination</p>
            <h3 id="link-details-title">Link details</h3>
          </div>
          <span class="item-id">ID {{ url_id }}</span>
        </header>

        <div class="link-content">
          <dl class="link-metadata">
            <div class="metadata-item destination-item">
              <dt>Destination URL</dt>
              <dd>{{ url_data.url || '—' }}</dd>
            </div>
            <div class="metadata-item">
              <dt>Short code</dt>
              <dd class="code-value">{{ url_data.code || '—' }}</dd>
            </div>
            <div class="metadata-item">
              <dt>Created</dt>
              <dd>{{ url_data.created_at ? humanReadable(url_data.created_at) : '—' }}</dd>
            </div>
            <div class="metadata-item">
              <dt>Type</dt>
              <dd class="type-value">
                <Icon
                  v-if="url_data.url_type === 'image'"
                  :icon="faImage"
                  aria-hidden="true"
                />
                <Icon
                  v-else-if="url_data.url_type === 'video'"
                  :icon="faVideo"
                  aria-hidden="true"
                />
                <Icon
                  v-else-if="url_data.url_type === 'url'"
                  :icon="faLink"
                  aria-hidden="true"
                />
                {{ url_data.url_type || '—' }}
              </dd>
            </div>
          </dl>

          <img
            v-if="url_data.url_type === 'image'"
            class="image-preview"
            :src="url_data.url"
            :alt="`Preview of ${url_data.code || 'short link'}`"
          />
        </div>
      </section>

      <section class="app-panel dashboard-panel activity-panel" aria-labelledby="activity-title">
        <header class="app-panel-heading panel-heading activity-heading">
          <div>
            <p class="app-kicker">Traffic</p>
            <h3 id="activity-title">Visit activity</h3>
          </div>
          <div class="record-count" aria-label="Visit records">
            <span class="accent-mark" aria-hidden="true" />
            <strong>{{ visit_data.length }}</strong>
            <span>records</span>
          </div>
        </header>

        <div class="activity-toolbar">
          <div class="view-tabs" role="tablist" aria-label="Visit views">
            <button
              class="view-tab"
              :class="{ 'is-active': tab === 'charts' }"
              type="button"
              role="tab"
              :aria-selected="tab === 'charts'"
              @click="tab = 'charts'"
            >
              Trend
            </button>
            <button
              class="view-tab"
              :class="{ 'is-active': tab === 'logs' }"
              type="button"
              role="tab"
              :aria-selected="tab === 'logs'"
              @click="tab = 'logs'"
            >
              Visit log
            </button>
          </div>

          <div v-if="tab === 'charts'" class="date-filters">
            <label class="filter-field">
              <span>Year</span>
              <select v-model="selectedYear" :disabled="!years.length">
                <option v-if="!years.length" :value="selectedYear">No dates</option>
                <option v-for="year in years" :key="year" :value="year">{{ year }}</option>
              </select>
            </label>
            <label class="filter-field">
              <span>Month</span>
              <select v-model="selectedMonth">
                <option :value="0">All months</option>
                <option v-for="month in monthOptions" :key="month.value" :value="month.value">
                  {{ month.label }}
                </option>
              </select>
            </label>
          </div>
        </div>

        <div v-if="tab === 'charts'" class="chart-container" role="tabpanel">
          <Chart
            :key="chartKey"
            type="line"
            height="320"
            :options="chartOptions"
            :series="series"
          />
        </div>

        <div v-else class="log-view" role="tabpanel">
          <label class="search-field">
            <Icon :icon="faSearch" aria-hidden="true" />
            <input
              v-model="searchQuery"
              aria-label="Search visit records"
              placeholder="Search IP, provider, country, or city"
              type="search"
            />
          </label>

          <div class="table-container">
            <table class="visits-table">
              <thead>
                <tr>
                  <th scope="col" class="row-number">#</th>
                  <th scope="col">Location</th>
                  <th scope="col">IP address</th>
                  <th scope="col" class="number-cell">Visits</th>
                  <th scope="col">Date and time</th>
                  <th scope="col" class="details-heading">Details</th>
                </tr>
              </thead>
              <tbody>
                <template v-for="(visit, index) in filteredVisits" :key="index">
                  <tr :class="{ 'is-expanded': visit.show_detail }">
                    <th scope="row" class="row-number">{{ index + 1 }}</th>
                    <td>
                      {{
                        [visit.geo_data?.city, visit.geo_data?.region, visit.country]
                          .filter(Boolean)
                          .join(', ') || '—'
                      }}
                    </td>
                    <td class="code-value">{{ visit.ip_address || '—' }}</td>
                    <td class="number-cell">{{ visit.count }}</td>
                    <td>{{ humanReadable(visit.action_at) }}</td>
                    <td class="details-cell">
                      <button
                        class="details-button"
                        type="button"
                        :aria-expanded="visit.show_detail"
                        @click="visit.show_detail = !visit.show_detail"
                      >
                        {{ visit.show_detail ? 'Hide' : 'View' }}
                      </button>
                    </td>
                  </tr>
                  <tr v-if="visit.show_detail" class="expanded-row">
                    <td colspan="6">
                      <dl class="visit-metadata">
                        <div>
                          <dt>Location</dt>
                          <dd>
                            {{
                              [visit.geo_data?.city, visit.geo_data?.region, visit.country]
                                .filter(Boolean)
                                .join(', ') || '—'
                            }}
                          </dd>
                        </div>
                        <div>
                          <dt>Provider</dt>
                          <dd>{{ visit.geo_data?.isp || '—' }}</dd>
                        </div>
                        <div>
                          <dt>Organization</dt>
                          <dd>{{ visit.geo_data?.org || '—' }}</dd>
                        </div>
                        <div>
                          <dt>Postcode</dt>
                          <dd>{{ visit.geo_data?.zip || '—' }}</dd>
                        </div>
                        <div>
                          <dt>User agent</dt>
                          <dd>{{ visit.user_agent || '—' }}</dd>
                        </div>
                      </dl>
                    </td>
                  </tr>
                  <tr v-if="visit.show_detail" class="map-row">
                    <td colspan="6">
                      <VisitMap
                        v-if="visit.geo_data?.lat != null && visit.geo_data?.lon != null"
                        :lat="visit.geo_data.lat"
                        :lon="visit.geo_data.lon"
                      />
                      <span v-else class="empty-location">No location data</span>
                    </td>
                  </tr>
                </template>
                <tr v-if="!filteredVisits.length">
                  <td colspan="6" class="empty-state">
                    {{ visit_data.length ? 'No records match this search' : 'No visit records yet' }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

<script setup>
import axios from "axios"
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue"
import dayjs from "dayjs"
import utc from "dayjs/plugin/utc"
import localizedFormat from "dayjs/plugin/localizedFormat"
import Chart from "vue3-apexcharts"
import { faSearch, faImage, faVideo, faLink } from "@fortawesome/free-solid-svg-icons"
import VisitMap from "@/components/VisitMap.vue"

dayjs.extend(utc)
dayjs.extend(localizedFormat)

const { url_id, url_code } = defineProps(["url_id", "url_code"])

const humanReadable = iso =>
  dayjs.utc(iso).local().format("DD MMMM YYYY HH:mm")

const visit_data = ref([])
const url_data = ref([])
const tab = ref("charts")

/* ------------------ FETCH ------------------ */
const get_url_visit_action_list = async () => {
  const r = await axios.get("/admin/analytical/list", {
    params: { url_id }
  })
  visit_data.value = r.data
  visit_data.value.forEach(v => (v.show_detail = false))
}

const get_url_data = async () => {
  const r = await axios.get("/admin/url", {
    params: { code: url_code }
  })
  url_data.value = r.data
}

const dates = computed(() =>
  visit_data.value.map(v => new Date(v.action_at))
)

const years = computed(() =>
  [...new Set(dates.value.map(d => d.getFullYear()))].sort((a, b) => a - b)
)
const monthOptions = Array.from({ length: 12 }, (_, index) => ({
  value: index + 1,
  label: dayjs().month(index).format("MMM")
}))
const today = new Date()

const selectedYear = ref(today.getFullYear())
// const selectedMonth = ref(today.getMonth() + 1)
const selectedMonth = ref(0)
const selectedDay = ref(null)

const chartOptions = ref({
  chart: {
    toolbar: { show: false },
    foreColor: "#a1a1a6",
    fontFamily: "inherit",
    background: "transparent",
    events: {
      dataPointSelection(_, ctx, { dataPointIndex }) {
        const x = ctx.opts.xaxis.categories[dataPointIndex]

        if (selectedMonth.value === 0) {
          selectedMonth.value = Number(x)
          return
        }

        if (!selectedDay.value) {
          selectedDay.value = Number(x)
        }
      }
    }
  },
  colors: ["#f29a62"],
  theme: {
    mode: "dark"
  },
  dataLabels: { enabled: false },
  stroke: { curve: "smooth", width: 2.5 },
  markers: { size: 0, hover: { size: 5 } },
  grid: { borderColor: "#343436", strokeDashArray: 4 },
  xaxis: {
    categories: [],
    axisBorder: { show: false },
    axisTicks: { show: false }
  },
  yaxis: { min: 0, forceNiceScale: true },
  tooltip: { theme: "dark" },
  noData: { text: "No visits in this period" },
  annotations: { xaxis: [] }
})

const chartKey = ref(0)

const applyChartTheme = () => {
  if (tab.value !== "charts") return
  const isDark = document.documentElement.getAttribute("data-theme") === "dark"
  const accent = isDark ? "#f29a62" : "#a84310"

  chartOptions.value = {
    ...chartOptions.value,
    chart: {
      ...chartOptions.value.chart,
      foreColor: isDark ? "#a1a1a6" : "#525252"
    },
    theme: {
      mode: isDark ? "dark" : "light"
    },
    colors: [accent],
    grid: {
      ...chartOptions.value.grid,
      borderColor: isDark ? "#343436" : "#e4e4e7"
    },
    tooltip: { theme: isDark ? "dark" : "light" }
  }

  chartKey.value++
}
watch(tab, val => {
  if (val === "charts") {
    nextTick(() => applyChartTheme())
  }
})

const series = ref([{ name: "Visits", data: [] }])

const updateChart = () => {
  selectedDay.value ??= null

  const filtered = dates.value.filter(d =>
    d.getFullYear() === selectedYear.value &&
    (selectedMonth.value === 0 ||
      d.getMonth() + 1 === selectedMonth.value)
  )

  if (!filtered.length) {
    chartOptions.value.xaxis.categories = []
    series.value = [{ name: "Visits", data: [] }]
    return
  }

  // HOURLY
  if (selectedDay.value) {
    const grouped = filtered
      .filter(d => d.getDate() === selectedDay.value)
      .reduce((a, d) => {
        const h = String(d.getHours()).padStart(2, "0")
        a[h] = (a[h] || 0) + 1
        return a
      }, {})

    const categories = Object.keys(grouped).sort()

    chartOptions.value.xaxis.categories = categories
    chartOptions.value.annotations.xaxis = []
    series.value = [{ name: "Visits", data: categories.map(c => grouped[c]) }]
    return
  }

  // DAILY / MONTHLY
  const grouped = filtered.reduce((a, d) => {
    const k =
      selectedMonth.value === 0
        ? String(d.getMonth() + 1).padStart(2, "0")
        : String(d.getDate()).padStart(2, "0")

    a[k] = (a[k] || 0) + 1
    return a
  }, {})

  const categories = Object.keys(grouped).sort()

  chartOptions.value.xaxis.categories = categories
  series.value = [{ name: "Visits", data: categories.map(c => grouped[c]) }]

  // TODAY LINE
  chartOptions.value.annotations.xaxis = []
  if (selectedMonth.value !== 0) {
    const todayDay = String(today.getDate()).padStart(2, "0")
    if (categories.includes(todayDay)) {
      const isDark = document.documentElement.getAttribute("data-theme") === "dark"
      const accent = isDark ? "#f29a62" : "#a84310"
      chartOptions.value.annotations.xaxis.push({
        x: todayDay,
        borderColor: accent,
        label: {
          text: "Today",
          style: {
            background: accent,
            color: isDark ? "#24150d" : "#ffffff"
          }
        }
      })
    }
  }
}

const searchQuery = ref("")

const filteredVisits = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return visit_data.value

  return visit_data.value.filter(v =>
    [
      v.ip_address,
      v.country,
      v.geo_data?.region,
      v.geo_data?.city,
      v.geo_data?.isp,
      v.geo_data?.org
    ]
      .filter(Boolean)
      .some(field =>
        String(field).toLowerCase().includes(query)
      )
  )
})

watch([visit_data, selectedYear, selectedMonth], updateChart)
watch([selectedYear, selectedMonth], () => (selectedDay.value = null))

watch(visit_data, () => {
  if (!years.value.length) return

  if (!years.value.includes(selectedYear.value)) {
    selectedYear.value = years.value[years.value.length - 1]
  }

  const hasSelectedMonth = dates.value.some(date =>
    date.getFullYear() === selectedYear.value &&
    date.getMonth() + 1 === selectedMonth.value
  )

  if (!hasSelectedMonth) selectedMonth.value = 0
})

const updateTheme = () => {
  applyChartTheme()
  updateChart()
}

const observer = new MutationObserver(updateTheme)

observer.observe(document.documentElement, {
  attributes: true,
  attributeFilter: ["data-theme"]
})

onBeforeUnmount(() => observer.disconnect())

onMounted(async () => {
  await get_url_visit_action_list()
  await get_url_data()
  applyChartTheme()
})
</script>

<style scoped>
.visit-dashboard {
  --page-surface: var(--app-page);
  --panel-surface: var(--app-surface);
  --raised-surface: var(--app-raised);
  --border-color: var(--app-border);
  --text-primary: var(--app-text);
  --text-secondary: var(--app-text-secondary);
  --text-muted: var(--app-text-muted);
  --accent: var(--app-accent);
  --accent-fill: var(--app-accent-fill);
  --accent-ink: var(--app-accent-ink);
  min-height: calc(100vh - 64px);
  color: var(--text-primary);
  background: var(--page-surface);
}

.view-tab:focus-visible,
.details-button:focus-visible,
.filter-field select:focus-visible,
.search-field input:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.dashboard-panel {
  width: 100%;
}

.link-panel {
  margin-bottom: 24px;
}

.item-id {
  color: var(--text-muted);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.link-content {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(220px, 0.4fr);
  gap: 24px;
  align-items: stretch;
}

.link-metadata {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px 24px;
  margin: 0;
}

.metadata-item {
  min-width: 0;
}

.metadata-item dt,
.visit-metadata dt {
  margin-bottom: 4px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 16px;
}

.metadata-item dd,
.visit-metadata dd {
  margin: 0;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 20px;
}

.destination-item {
  grid-column: 1 / -1;
}

.destination-item dd {
  overflow-wrap: anywhere;
}

.code-value {
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-variant-numeric: tabular-nums;
}

.type-value {
  display: flex;
  align-items: center;
  gap: 8px;
  text-transform: capitalize;
}

.image-preview {
  display: block;
  width: 100%;
  min-height: 160px;
  max-height: 220px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background-color: var(--raised-surface);
  object-fit: contain;
}

.activity-panel {
  margin-bottom: 24px;
}

.activity-heading {
  margin-bottom: 16px;
}

.record-count {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 13px;
}

.record-count strong {
  color: var(--text-primary);
  font-size: 15px;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.accent-mark {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
}

.activity-toolbar {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  border-bottom: 1px solid var(--border-color);
}

.view-tabs {
  display: flex;
  align-items: center;
  gap: 20px;
}

.view-tab {
  min-height: 40px;
  padding: 0 2px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font: inherit;
  font-size: 14px;
}

.view-tab:hover {
  color: var(--text-primary);
}

.view-tab.is-active {
  border-bottom-color: var(--accent);
  color: var(--text-primary);
}

.date-filters {
  display: flex;
  gap: 12px;
  padding-bottom: 8px;
}

.filter-field {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 12px;
}

.filter-field select {
  min-width: 96px;
  height: 36px;
  padding: 0 28px 0 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--raised-surface);
  color: var(--text-primary);
  font: inherit;
  font-size: 13px;
}

.filter-field select:disabled {
  cursor: not-allowed;
  opacity: 0.65;
}

.chart-container {
  padding-top: 16px;
}

.log-view {
  padding-top: 20px;
}

.search-field {
  display: flex;
  width: min(100%, 360px);
  height: 40px;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--raised-surface);
  color: var(--text-muted);
}

.search-field input {
  width: 100%;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
  font: inherit;
  font-size: 13px;
}

.search-field input::placeholder {
  color: var(--text-muted);
}

.table-container {
  overflow-x: auto;
}

.visits-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 13px;
}

.visits-table th,
.visits-table td {
  padding: 12px 10px;
  border-bottom: 1px solid var(--border-color);
  vertical-align: middle;
}

.visits-table thead th {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.visits-table tbody tr:not(.expanded-row, .map-row):hover {
  background: var(--raised-surface);
}

.row-number {
  width: 40px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.number-cell {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.details-heading,
.details-cell {
  text-align: right;
}

.details-button {
  min-width: 52px;
  min-height: 32px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--panel-surface);
  color: var(--text-secondary);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
}

.details-button:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.expanded-row td,
.map-row td {
  padding: 16px;
  background: var(--raised-surface);
}

.visit-metadata {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
  margin: 0;
}

.visit-metadata dd {
  overflow-wrap: anywhere;
}

.map-row td {
  padding-top: 0;
}

.empty-location {
  color: var(--text-muted);
  font-size: 13px;
}

.empty-state {
  height: 96px;
  color: var(--text-muted);
  text-align: center;
}

@media (max-width: 720px) {
  .link-content {
    grid-template-columns: 1fr;
  }

  .image-preview {
    min-height: 180px;
  }

  .activity-toolbar {
    align-items: flex-start;
    flex-direction: column;
    gap: 12px;
  }

  .date-filters {
    width: 100%;
    padding-bottom: 12px;
  }

  .filter-field {
    flex: 1;
  }

  .filter-field select {
    min-width: 0;
    width: 100%;
  }

  .visit-metadata {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 480px) {
  .edit-link {
    width: 100%;
  }

  .link-metadata {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .destination-item {
    grid-column: auto;
  }

  .activity-heading {
    align-items: flex-start;
  }

  .record-count {
    padding-top: 4px;
  }

  .filter-field {
    align-items: flex-start;
    flex-direction: column;
    gap: 4px;
  }

  .visit-metadata {
    grid-template-columns: 1fr;
  }

  .visits-table {
    table-layout: fixed;
  }

  .visits-table thead th:first-child,
  .visits-table thead th:nth-child(2),
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :first-child,
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :nth-child(2) {
    display: none;
  }

  .visits-table thead th:nth-child(3),
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :nth-child(3) {
    width: 32%;
  }

  .visits-table thead th:nth-child(4),
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :nth-child(4) {
    width: 14%;
  }

  .visits-table thead th:nth-child(5),
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :nth-child(5) {
    width: 34%;
  }

  .visits-table thead th:nth-child(6),
  .visits-table tbody tr:not(.expanded-row):not(.map-row) > :nth-child(6) {
    width: 20%;
  }

  .visits-table th,
  .visits-table td {
    padding: 10px 6px;
    overflow-wrap: anywhere;
  }

  .visits-table tbody tr:not(.expanded-row, .map-row) > td:nth-child(5) {
    font-size: 11px;
  }
}
</style>
