<script setup lang="ts">
import {MangaData, SearchData} from "../bindings.ts";
import {ref} from "vue";
import SearchPane from "./DownloadComponents/SearchPane.vue";
import EpisodePane from "./DownloadComponents/EpisodePane.vue";
import DownloadingList from "./DownloadComponents/DownloadingList.vue";

const searchData = ref<SearchData>();
const mangaData = ref<MangaData>();

async function test() {
  console.log(searchData.value);
  console.log(mangaData.value);
}


</script>

<template>
  <div class="h-full flex flex-col">
    <n-button @click="test">测试用</n-button>
    <div class="flex flex-1 overflow-hidden">
      <div class="basis-1/2 overflow-auto">
        <n-tabs default-value="search" type="line" animated size="small" class="h-full">
          <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show:lazy">
            <search-pane v-model:search-data="searchData" v-model:manga-data="mangaData">
            </search-pane>
          </n-tab-pane>
          <n-tab-pane class="h-full overflow-auto p-0!" name="download" tab="章节详情" display-directive="show:lazy">
            <episode-pane></episode-pane>
          </n-tab-pane>
        </n-tabs>
      </div>
      <div class="basis-1/2 overflow-auto">
        <downloading-list class="h-full"></downloading-list>
      </div>
    </div>
  </div>
</template>
