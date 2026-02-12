<template>
  <section class="page-grid hardware-page">
    <article class="glass-panel page-header">
      <h2>{{ t('hardware.title') }}</h2>
      <p>{{ t('hardware.subtitle') }}</p>
    </article>

    <article class="glass-panel hardware-tile">
      <header>
        <span class="material-symbols-outlined">computer</span>
        <small>{{ t('hardware.processor') }}</small>
      </header>
      <h3>{{ info.cpu_model }}</h3>
      <p>{{ cpuFreqLabel }} · {{ cpuTemp }}</p>
    </article>

    <article class="glass-panel hardware-tile">
      <header>
        <span class="material-symbols-outlined">videogame_asset</span>
        <small>{{ t('hardware.graphics') }}</small>
      </header>
      <h3>{{ info.gpu_model }}</h3>
      <p>
        {{
          snapshot.gpu.usage_pct == null
            ? t('common.na')
            : `${snapshot.gpu.usage_pct.toFixed(1)}% ${t('hardware.load')}`
        }}
      </p>
    </article>

    <article class="glass-panel hardware-tile">
      <header>
        <span class="material-symbols-outlined">memory_alt</span>
        <small>{{ t('hardware.memory') }}</small>
      </header>
      <h3>{{ info.ram_spec }}</h3>
      <p>
        {{ (snapshot.memory.used_mb / 1024).toFixed(1) }} / {{ (snapshot.memory.total_mb / 1024).toFixed(1) }} GB
      </p>
    </article>

    <article class="glass-panel hardware-tile">
      <header>
        <span class="material-symbols-outlined">developer_board</span>
        <small>{{ t('hardware.mainboard') }}</small>
      </header>
      <h3>{{ info.motherboard }}</h3>
      <p>{{ info.device_brand }}</p>
    </article>

    <article class="glass-panel full-width hardware-list">
      <h3>{{ t('hardware.storage') }}</h3>
      <ul>
        <li v-for="disk in snapshot.disks" :key="disk.name">
          <span class="material-symbols-outlined">database</span>
          <div>
            <strong>
              {{ disk.name }}
              <small>{{ disk.label }}</small>
            </strong>
            <p>
              {{ disk.usage_pct.toFixed(1) }}% · {{ disk.used_gb.toFixed(1) }} / {{ disk.total_gb.toFixed(1) }} GB
            </p>
            <p style="font-size: 0.8em; opacity: 0.7">
              R: {{ ((disk.read_bytes_per_sec || 0) / 1024 / 1024).toFixed(1) }} MB/s · W:
              {{ ((disk.write_bytes_per_sec || 0) / 1024 / 1024).toFixed(1) }} MB/s
            </p>
          </div>
        </li>
      </ul>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

import { useAppStore } from '../stores/app';

const { t } = useI18n();
const store = useAppStore();
const info = computed(() => store.hardwareInfo);
const snapshot = computed(() => store.snapshot);

const cpuTemp = computed(() => {
  const value = snapshot.value.cpu.temperature_c;
  return value == null ? t('common.na') : `${value.toFixed(1)} °C`;
});

const cpuFreqLabel = computed(() => {
  const freq = snapshot.value.cpu.frequency_mhz;
  const max = info.value.cpu_max_freq_mhz;

  if (freq == null) {
    return t('common.na');
  }

  if (max) {
    return `${(freq / 1000).toFixed(1)}/${(max / 1000).toFixed(1)} GHz`;
  }

  return `${(freq / 1000).toFixed(1)} GHz`;
});
</script>
