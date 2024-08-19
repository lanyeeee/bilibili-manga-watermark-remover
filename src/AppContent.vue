<script setup lang="ts">


import WatermarkPane from "./components/WatermarkPane.vue";
import DownloadPane from "./components/DownloadPane.vue";
import {onMounted} from "vue";

onMounted(() => {
  document.oncontextmenu = (event) => {
    event.preventDefault();
  };
});
</script>

<template>
  <n-tabs default-value="download" type="line" animated class="h-screen">
    <n-tab-pane class="h-full overflow-auto p-0!" name="download" tab="去水印" display-directive="show:lazy">
      <watermark-pane></watermark-pane>
    </n-tab-pane>
    <n-tab-pane class="h-full overflow-auto p-0!" name="export" tab="下载" display-directive="show:lazy">
      <download-pane></download-pane>
    </n-tab-pane>
  </n-tabs>

</template>
