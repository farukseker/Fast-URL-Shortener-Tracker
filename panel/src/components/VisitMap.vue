<template>
  <div ref="mapEl" class="visit-map" role="region" aria-label="Interactive visit location map"></div>
</template>

<script setup>
import { onMounted, ref } from "vue"
import L from "leaflet"
import "leaflet/dist/leaflet.css"

const props = defineProps({
  lat: Number,
  lon: Number
})

const mapEl = ref(null)

onMounted(() => {
  const map = L.map(mapEl.value).setView([props.lat, props.lon], 10)
  const markerIcon = L.divIcon({
    className: "visit-map-marker",
    html: "<span></span>",
    iconSize: [18, 18],
    iconAnchor: [9, 9]
  })

  L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 18
  }).addTo(map)

  L.marker([props.lat, props.lon], { icon: markerIcon }).addTo(map)
})
</script>

<style>
.visit-map {
  width: 100%;
  height: 256px;
  overflow: hidden;
  border: 1px solid var(--app-border);
  border-radius: 10px;
}

.visit-map-marker {
  border: 0;
  background: transparent;
}

.visit-map-marker::before {
  display: block;
  width: 12px;
  height: 12px;
  border: 2px solid #ffffff;
  border-radius: 50%;
  background: var(--app-accent-fill);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  content: "";
}
</style>