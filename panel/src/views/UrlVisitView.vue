<template>

<section class="w-full flex justify-center h-screen absolute top-0 left-0 pt-10 mx-auto overflow-auto ">
  <article class="flex flex-col gap-4 p-4 w-full md:w-2/3 mt-10">
    <fieldset class="fieldset bg-base-200 rounded-box w-full p-4 shadow-xl border border-primary">
        <legend class="fieldset-legend">Url details ({{ url_id }})</legend>
        <div class="gird flex grid-cols-2 h-60">
            <div class="text-lg w-full">
                <div class="mb-2">
                    <RouterLink
                    class="btn btn-primary"
                    :to="{ name: 'manage-url', params: { url_code: url_data.code } }"
                    >
                    Edit
                    </RouterLink>
                </div>
                <ul>
                    <li><strong>Url:</strong> {{ url_data.url }}</li>
                    <li><strong>Code:</strong> {{ url_data.code }}</li>
                    <li><strong>Created:</strong>  {{ humanReadable(url_data.created_at) }}</li>
                    <li>
                        <strong>Type: </strong>
                        <div class="tooltip tooltip-bottom tooltip-primary" data-tip="Image"><Icon v-if="url_data.url_type === 'image'" :icon="faImage" /></div> 
                        <div class="tooltip tooltip-bottom tooltip-primary" data-tip="Video"><Icon v-if="url_data.url_type === 'video'" :icon="faVideo" /></div> 
                        <div class="tooltip tooltip-bottom tooltip-primary" data-tip="Url"><Icon v-if="url_data.url_type === 'url'" :icon="faLink" /></div>
                    </li>
                </ul>
            </div>
            <div class="rounded shadow bg-base-300 w-1/2 flex justify-center">
                <span v-if="url_data.url_type !== 'image'" class="m-auto">Coming soon</span>
                <div
                v-else
                class="bg-no-repeat bg-center h-full w-full"
                :style="`background-image: url(${url_data.url}); background-size: contain;`">
                </div>
            </div>
        </div>
    </fieldset>

    <hr class="mt-3">

    <fieldset class="fieldset bg-base-200 rounded-box w-full p-4 shadow-xl border border-primary mb-20">
        <legend class="fieldset-legend">Visit details ({{ visit_data.length }})</legend>
        <div role="tablist" class="tabs tabs-border">
            <a role="tab" class="tab" :class="tab === 'charts' ? 'tab-active':''" @click="tab='charts'">Chart's</a>
            <a role="tab" class="tab" :class="tab === 'logs' ? 'tab-active':''" @click="tab='logs'">Log's</a>
        </div>
        <div v-if="tab === 'charts'" class="flex flex-col gap-4">
            <div class="flex gap-2">
                <select v-model="selectedYear" class="select select-bordered select-sm">
                <option v-for="y in years" :key="y" :value="y">{{ y }}</option>
                </select>

                <select v-model="selectedMonth" class="select select-bordered select-sm">
                <option :value="0">All</option>
                <option v-for="m in 12" :key="m" :value="m">
                    {{ String(m).padStart(2,'0') }}
                </option>
                </select>
            </div>
            <Chart
              :key="chartKey"
              type="line"
              height="350"
              :options="chartOptions"
              :series="series"
            />
        </div>
        <div v-if="tab === 'logs'">
            <label class="input w-full border-0 border-b shadow outline-0 my-4">
                <Icon :icon="faSearch" />
                <input
                  v-model="searchQuery"
                  class="grow"
                  placeholder="IP | Provider | Country | City"
                  type="text"
                />
            </label>
            <div class="overflow-x-auto">
                <table class="table table-zebra">
                <thead class="sticky top-0 z-10 bg-base-100">
                    <tr>
                        <th></th>
                        <th class="text-center">Country</th>
                        <th class="text-center">City</th>
                        <th class="text-center">Ip Address</th>
                        <th class="text-center">Count</th>
                        <th class="text-center">Date-Time</th>
                        <th class="text-center">Edit</th>
                        <!-- <th class="text-center">Delete</th> -->
                    </tr>
                </thead>
                <tbody>
                    <template v-for="(visit, index) in filteredVisits" :key="index">
                    <tr class="text-center" :class="2%index === 0 ? 'bg-base-300': ''">
                        <th>{{ index + 1 }}</th>
                        <td>{{ visit.country ? visit.country:'-' }}</td>
                        <td>{{ visit.geo_data?.region ? visit.geo_data?.region:'-' }}/{{ visit.geo_data?.city ? visit.geo_data?.city:'-' }}</td>
                        <td>{{ visit.ip_address ? visit.ip_address:'-' }}</td>
                        <td>{{ visit.count }}</td>
                        <td>{{ humanReadable(visit.action_at) }}</td>
                        <td @click="visit.show_detail =! visit.show_detail" class="min-w-32">
                            <span class="">{{ visit.show_detail ? 'Hide':'Details'}}</span>
                        </td>
                    </tr>
                    <template v-if="visit.show_detail">
                      <tr>
                          <td class="shadow" :colspan="7">
                              <ul>
                                  <li><strong>Provider: </strong>{{ visit.geo_data?.isp || '-' }}</li>
                                  <li><strong>Provider Name: </strong>{{ visit.geo_data?.org || '-' }}</li>
                                  <li><strong>Post Code: </strong>{{ visit.geo_data?.zip || '-' }}</li>
                                  <li><strong>User Agent: </strong>{{visit.user_agent}}</li>
                              </ul> 
                          </td>
                      </tr>
                      <tr>
                          <td class="shadow" :colspan="7">
                            <VisitMap
                              v-if="visit.geo_data?.lat && visit.geo_data?.lon"
                              :lat="visit.geo_data.lat"
                              :lon="visit.geo_data.lon"
                            />
                            <span v-else>- No geo location -</span>
                          </td>
                      </tr>
                    </template>
                  </template>
                </tbody>
                </table>
            </div>
        </div>
    </fieldset>

    <div class="min-h-4"></div>
  </article>
</section>
</template>

<script setup>
import axios from "axios"
import { ref, computed, watch, onMounted, nextTick } from "vue"
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

const groupBy = (arr, fn) =>
  arr.reduce((a, c) => {
    const k = fn(c)
    a[k] = (a[k] || 0) + 1
    return a
  }, {})

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
  [...new Set(dates.value.map(d => d.getFullYear()))].sort()
)
const today = new Date()

const selectedYear = ref(today.getFullYear())
const selectedMonth = ref(today.getMonth() + 1) // 0 = all
const selectedDay = ref(null)

const isDarkTheme = () =>  document.body.getAttribute("data-theme") === "dark"

const chartOptions = ref({
  chart: {
    toolbar: { show: false },
    foreColor: "#374151",
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
  theme: {
    mode: "light"
  },
  stroke: { curve: "smooth" },
  xaxis: { categories: [] },
  annotations: { xaxis: [] }
})

const chartKey = ref(0)

const applyChartTheme = () => {
  if (tab.value !== "charts") return
  const isDark = document.documentElement.getAttribute("data-theme") === "dark"

  chartOptions.value = {
    ...chartOptions.value,
    chart: {
      ...chartOptions.value.chart,
      foreColor: isDark ? "#e5e7eb" : "#374151"
    },
    theme: {
      mode: isDark ? "dark" : "light"
    }
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
      chartOptions.value.annotations.xaxis.push({
        x: todayDay,
        borderColor: "#ef4444",
        label: { text: "Today" }
      })
    }
  }
}

const searchQuery = ref("")

const filteredVisits = computed(() => {
  if (!searchQuery.value) return visit_data.value

  const q = searchQuery.value.toLowerCase()

  return visit_data.value.filter(v =>
    [
      v.ip_address,
      v.ip_provider,
      v.country,
      v.city
    ]
      .filter(Boolean)
      .some(field =>
        field.toLowerCase().includes(q)
      )
  )
})

watch([visit_data, selectedYear, selectedMonth], updateChart)
watch([selectedYear, selectedMonth], () => (selectedDay.value = null))


const updateChartTheme = () => {
  chartOptions.value.foreColor =
    isDarkTheme() ? "#e5e7eb" : "#374151"
}

const observer = new MutationObserver(applyChartTheme)

observer.observe(document.body, {
  attributes: true,
  attributeFilter: ["data-theme"]
})


onMounted(async () => {
  await get_url_visit_action_list()
  await get_url_data()
  applyChartTheme()
})
</script>
