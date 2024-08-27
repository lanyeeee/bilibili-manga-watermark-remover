<script setup lang="ts">
import {useMessage, useNotification} from "naive-ui";


import WatermarkPane from "./components/WatermarkPane.vue";
import DownloadPane from "./components/DownloadPane.vue";
import {onMounted, ref, watch} from "vue";
import {commands, Config} from "./bindings.ts";

const message = useMessage();
const notification = useNotification();

const config = ref<Config>();

watch(config, async () => {
  if (config.value === undefined) {
    return;
  }
  const result = await commands.saveConfig(config.value);
  if (result.status === "error") {
    notification.error({title: "保存配置失败", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "保存配置失败", description: response.msg});
    return;
  }
  message.success("保存配置成功");
}, {deep: true});

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault();
  };
  // 获取配置
  const response = await commands.getConfig();
  if (response.code !== 0) {
    notification.warning({title: "获取配置失败", description: response.msg});
    return;
  }
  config.value = response.data;
});
</script>

<template>
  <n-tabs default-value="download" type="line" animated class="h-screen">
    <n-tab-pane class="h-full overflow-auto p-0!" name="download" tab="去水印" display-directive="show:lazy">
      <watermark-pane v-model:config="config"></watermark-pane>
    </n-tab-pane>
    <n-tab-pane class="h-full overflow-auto p-0!" name="export" tab="下载" display-directive="show:lazy">
      <download-pane v-model:config="config"></download-pane>
    </n-tab-pane>
  </n-tabs>

</template>
