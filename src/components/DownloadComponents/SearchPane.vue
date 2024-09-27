<script setup lang="ts">
import {ref} from "vue";
import {commands, Episode, SearchData} from "../../bindings.ts";
import {useNotification} from "naive-ui";
import SearchResult from "./SearchResult.vue";

const notification = useNotification();

const searchData = defineModel<SearchData | undefined>("searchData", {required: true});
const currentTabName = defineModel<"search" | "episode">("currentTabName", {required: true});
const mangaId = defineModel<number | undefined>("mangaId", {required: true});
const episodes = defineModel<Episode[] | undefined>("episodes", {required: true});

const searchInput = ref("");
const mangaIdInput = ref("");

async function searchByKeyword(keyword: string) {
  let result = await commands.searchManga(keyword);
  if (result.status === "error") {
    notification.error({title: "搜索失败", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "搜索失败", description: response.msg});
    return;
  }
  searchData.value = response.data;
  console.log("searchData", searchData.value);
}

async function searchById(id: number) {
  let result = await commands.getMangaEpisodes(id);
  if (result.status === "error") {
    notification.error({title: "获取漫画章节详情失败", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "获取漫画章节详情失败", description: response.msg});
    return;
  }
  episodes.value = response.data;
  mangaId.value = id;
  // 切换到章节详情页
  currentTabName.value = "episode";
}

</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex flex-1">
      <n-input class="text-align-left"
               size="tiny"
               v-model:value="searchInput"
               placeholder=""
               clearable
               @keydown.enter="searchByKeyword(searchInput.trim())"
      >
        <template #prefix>
          漫画名:
        </template>
      </n-input>
      <n-button size="tiny" @click="searchByKeyword(searchInput.trim())">搜索</n-button>
      <div class="min-w-2"></div>
      <n-input class="text-align-left"
               size="tiny"
               v-model:value="mangaIdInput"
               placeholder=""
               clearable
               @keydown.enter="searchById(Number(mangaIdInput.trim()))"
      >
        <template #prefix>
          漫画ID:
        </template>
      </n-input>
      <n-button size="tiny" @click="searchById(Number(mangaIdInput.trim()))">直达</n-button>
    </div>
    <search-result :on-click-item="searchById" :search-data="searchData"/>
  </div>
</template>