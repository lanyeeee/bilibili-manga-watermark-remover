<script setup lang="ts">


import WatermarkPane from "./components/WatermarkPane.vue";
import DownloadPane from "./components/DownloadPane.vue";
</script>

<template>
  <n-tabs default-value="download" type="line" animated class="h-screen p-3">
    <n-tab-pane class="h-full p-0!" name="download" tab="去水印" display-directive="show:lazy">
      <watermark-pane></watermark-pane>
    </n-tab-pane>
    <n-tab-pane class="h-full p-0!" name="export" tab="下载" display-directive="show:lazy">
      <download-pane></download-pane>
    </n-tab-pane>
  </n-tabs>

</template>
