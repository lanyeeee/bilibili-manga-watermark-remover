<script setup lang="ts">
import {Config, MangaData, SearchData} from "../bindings.ts";
import {ref, watch} from "vue";
import SearchPane from "./DownloadComponents/SearchPane.vue";
import EpisodePane from "./DownloadComponents/EpisodePane.vue";
import DownloadingList from "./DownloadComponents/DownloadingList.vue";
import QrCodeViewer from "./DownloadComponents/QrCodeViewer.vue";

const config = defineModel<Config | undefined>("config", {required: true});

const biliCookie = ref<string>(config.value?.biliCookie ?? "");
const qrCodeViewerShowing = ref<boolean>(false);
const searchData = ref<SearchData>();
const mangaData = ref<MangaData>();

watch(biliCookie, (value) => {
  if (config.value === undefined) {
    return;
  }
  config.value.biliCookie = value;
});

async function test() {
  console.log(searchData.value);
  console.log(mangaData.value);
}


</script>

<template>
  <div class="h-full flex flex-col">
    <n-button @click="test">测试用</n-button>
    <div class="flex">
      <n-input v-model:value="biliCookie" placeholder="" clearable>
        <template #prefix>
          Cookie:
        </template>
      </n-input>
      <n-button @click="qrCodeViewerShowing=true">二维码登录</n-button>
      <n-button>检测</n-button>
    </div>
    <div class="flex flex-1 overflow-hidden">
      <div class="basis-1/2 overflow-auto">
        <n-tabs default-value="search" type="line" animated size="small" class="h-full">
          <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show:lazy">
            <search-pane v-model:search-data="searchData" v-model:manga-data="mangaData">
            </search-pane>
          </n-tab-pane>
          <n-tab-pane class="h-full overflow-auto p-0!" name="download" tab="章节详情" display-directive="show:lazy">
            <episode-pane v-model:manga-data="mangaData"></episode-pane>
          </n-tab-pane>
        </n-tabs>
      </div>
      <div class="basis-1/2 overflow-auto">
        <downloading-list class="h-full"></downloading-list>
      </div>
    </div>
  </div>
  <n-modal preset="dialog" title="请使用BiliBili手机客户端扫描二维码登录" v-model:show="qrCodeViewerShowing">
    <qr-code-viewer v-model:showing="qrCodeViewerShowing" v-model:bili-cookie="biliCookie"/>
  </n-modal>
</template>
