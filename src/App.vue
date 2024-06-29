<script setup lang="ts">
import WatermarkPicker from "./components/WatermarkPicker.vue";
</script>

<template>
  <div>
    <watermark-picker :is-black="true"/>
    <watermark-picker :is-black="false"/>
  </div>
</template>
