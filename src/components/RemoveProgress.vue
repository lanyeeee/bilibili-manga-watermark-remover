<script setup lang="ts">
import {computed} from "vue";

const props = defineProps<{
  removeWatermarkTasks: Map<string, [number, number]>;
}>();

const tasksProgress = computed(() =>
    Array.from(props.removeWatermarkTasks)
        .map(([dirPath, [current, total]]) => ({
          dirPath,
          percentage: Math.round(current / total * 100),
        }))
);
</script>

<template>
  <div>
    <div v-for="status in tasksProgress" :key="status.dirPath">
      <n-progress :percentage="status.percentage">{{ status.percentage }}% {{ status.dirPath }}</n-progress>
    </div>
  </div>
</template>
