<template>
  <main class="app-page app-page--medium">
    <header class="app-page-heading">
      <div>
        <p class="app-kicker">Chart preview</p>
        <h2>Visit trend</h2>
        <p class="app-description">Explore the trend by year or month</p>
      </div>
    </header>

    <section class="app-panel trend-panel" aria-label="Visit trend chart">
      <div class="trend-filters">
        <label class="app-field">
          <span>Year</span>
          <span class="app-control">
            <select v-model="selectedYear">
              <option v-for="year in years" :key="year" :value="year">{{ year }}</option>
            </select>
          </span>
        </label>

        <label class="app-field">
          <span>Month</span>
          <span class="app-control">
            <select v-model="selectedMonth">
              <option value="all">All months</option>
              <option v-for="month in 12" :key="month" :value="month">
                {{ String(month).padStart(2, '0') }}
              </option>
            </select>
          </span>
        </label>
      </div>

      <Chart type="line" height="350" :options="chartOptions" :series="series" />
    </section>
  </main>
</template>

<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from "vue"

/* ------------------ FAKE ORDER DATA ------------------ */
const rawData = [
  { action_at: "2023-02-22T10:15:06+00:00" },
  { action_at: "2023-02-25T14:30:12+00:00" },

  { action_at: "2024-02-22T09:05:06+00:00" },

  { action_at: "2025-02-22T08:10:06+00:00" },
  { action_at: "2025-02-25T04:30:07+00:00" },

  { action_at: "2025-12-25T07:15:07+00:00" },
  { action_at: "2025-12-25T11:15:51+00:00" }
]

const dates = rawData.map(i => new Date(i.action_at))

const years = [...new Set(dates.map(d => d.getFullYear()))].sort()

const today = new Date()

const selectedYear = ref(years[years.length - 1] ?? today.getFullYear())
const selectedMonth = ref("all")

const chartOptions = ref({
  chart: {
    toolbar: { show: false },
    foreColor: "#a1a1a6",
    background: "transparent"
  },
  colors: ["#f29a62"],
  theme: { mode: "dark" },
  dataLabels: { enabled: false },
  stroke: { curve: "smooth", width: 2.5 },
  markers: { size: 0, hover: { size: 5 } },
  grid: { borderColor: "#343436", strokeDashArray: 4 },
  xaxis: { categories: [], axisBorder: { show: false }, axisTicks: { show: false } },
  tooltip: { theme: "dark" }
})

const series = ref([{ name: "Orders", data: [] }])

const groupBy = (arr, keyFn) =>
  arr.reduce((acc, item) => {
    const k = keyFn(item)
    acc[k] = (acc[k] || 0) + 1
    return acc
  }, {})

const updateChart = () => {
  const filtered = dates.filter(d =>
    d.getFullYear() === selectedYear.value &&
    (selectedMonth.value === "all" ||
      d.getMonth() + 1 === Number(selectedMonth.value))
  )

  // AUTO TODAY FALLBACK
  if (!filtered.length && selectedMonth.value !== "all") {
    selectedMonth.value = "all"
    return
  }

  let grouped

  if (selectedMonth.value === "all") {
    // MONTHLY
    grouped = groupBy(filtered, d =>
      String(d.getMonth() + 1).padStart(2, "0")
    )
  } else {
    // DAILY
    grouped = groupBy(filtered, d =>
      String(d.getDate()).padStart(2, "0")
    )
  }

  const categories = Object.keys(grouped)
    .map(Number)
    .sort((a, b) => a - b)
    .map(n => String(n).padStart(2, "0"))

  series.value = [
    {
      name: "Orders",
      data: categories.map(c => grouped[c])
    }
  ]

  chartOptions.value.xaxis.categories = categories
}

const applyChartTheme = () => {
  const isDark = document.documentElement.getAttribute("data-theme") === "dark"
  const accent = isDark ? "#f29a62" : "#a84310"

  chartOptions.value = {
    ...chartOptions.value,
    chart: {
      ...chartOptions.value.chart,
      foreColor: isDark ? "#a1a1a6" : "#525252"
    },
    colors: [accent],
    theme: { mode: isDark ? "dark" : "light" },
    grid: {
      ...chartOptions.value.grid,
      borderColor: isDark ? "#343436" : "#e4e4e7"
    },
    tooltip: { theme: isDark ? "dark" : "light" }
  }
}

const themeObserver = new MutationObserver(applyChartTheme)

onMounted(() => {
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"]
  })
  applyChartTheme()
})

onBeforeUnmount(() => themeObserver.disconnect())

watch([selectedYear, selectedMonth], updateChart, { immediate: true })
</script>

<style scoped>
.trend-panel {
  display: grid;
  gap: 16px;
}

.trend-filters {
  display: flex;
  gap: 16px;
}

.trend-filters .app-control {
  min-width: 130px;
}

@media (max-width: 480px) {
  .trend-filters {
    flex-direction: column;
  }
}
</style>
