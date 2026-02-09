<template>
  <section class="page-grid">
    <article class="glass-panel page-header">
      <h2>{{ t("dashboard.title") }}</h2>
      <p>500ms sampling · low-power fallback · best-effort temperature</p>
    </article>

    <template v-if="settings.module_toggles.show_cpu">
      <MetricCard
        :title="t('dashboard.cpu')"
        :value="`${snapshot.cpu.usage_pct.toFixed(1)}%`"
        :subtitle="`${snapshot.cpu.frequency_mhz ?? 'N/A'} MHz · ${temp(snapshot.cpu.temperature_c)}`"
        :percent="snapshot.cpu.usage_pct"
      />
    </template>

    <template v-if="settings.module_toggles.show_gpu">
      <MetricCard
        :title="t('dashboard.gpu')"
        :value="snapshot.gpu.usage_pct == null ? 'N/A' : `${snapshot.gpu.usage_pct.toFixed(1)}%`"
        :subtitle="`VRAM ${snapshot.gpu.memory_used_mb ?? 'N/A'} / ${snapshot.gpu.memory_total_mb ?? 'N/A'} MB · ${temp(snapshot.gpu.temperature_c)}`"
        :percent="snapshot.gpu.usage_pct ?? 0"
      />
    </template>

    <template v-if="settings.module_toggles.show_memory">
      <MetricCard
        :title="t('dashboard.memory')"
        :value="`${snapshot.memory.usage_pct.toFixed(1)}%`"
        :subtitle="`${snapshot.memory.used_mb.toFixed(0)} / ${snapshot.memory.total_mb.toFixed(0)} MB`"
        :percent="snapshot.memory.usage_pct"
      />
    </template>

    <template v-if="settings.module_toggles.show_disk">
      <MetricCard
        :title="t('dashboard.disk')"
        :value="`${snapshot.disk.usage_pct.toFixed(1)}%`"
        :subtitle="`${snapshot.disk.used_gb.toFixed(1)} / ${snapshot.disk.total_gb.toFixed(1)} GB`"
        :percent="snapshot.disk.usage_pct"
      />
    </template>

    <template v-if="settings.module_toggles.show_network">
      <MetricCard
        :title="t('dashboard.network')"
        :value="`${(snapshot.network.download_bytes_per_sec / 1024 / 1024).toFixed(2)} MB/s`"
        :subtitle="`Up ${(snapshot.network.upload_bytes_per_sec / 1024 / 1024).toFixed(2)} MB/s · ${snapshot.network.latency_ms ?? 'N/A'} ms`"
        :percent="Math.min(100, (snapshot.network.download_bytes_per_sec / 1024 / 1024) * 10)"
      />
    </template>

    <article v-if="settings.module_toggles.show_cpu" class="glass-panel chart-card">
      <header><h3>CPU %</h3></header>
      <LineChart :values="store.cpuHistory" color="#00f3ff" />
    </article>

    <article v-if="settings.module_toggles.show_memory" class="glass-panel chart-card">
      <header><h3>Memory %</h3></header>
      <LineChart :values="store.memoryHistory" color="#bc13fe" />
    </article>

    <article v-if="settings.module_toggles.show_network" class="glass-panel chart-card">
      <header><h3>Download MB/s</h3></header>
      <LineChart :values="store.networkDownHistory" color="#2b6cee" />
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import MetricCard from "../components/MetricCard.vue";
import LineChart from "../components/LineChart.vue";
import { useAppStore } from "../stores/app";

const store = useAppStore();
const { t } = useI18n();

const snapshot = computed(() => store.snapshot);
const settings = computed(() => store.settings);

function temp(value: number | null): string {
  return value == null ? "N/A" : `${value.toFixed(1)}°C`;
}
</script>
