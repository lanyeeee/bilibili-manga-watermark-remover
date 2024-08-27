<script setup lang="ts">
import {commands, Config, Episode, SearchData} from "../bindings.ts";
import {ref, watch} from "vue";
import SearchPane from "./DownloadComponents/SearchPane.vue";
import EpisodePane from "./DownloadComponents/EpisodePane.vue";
import DownloadingList from "./DownloadComponents/DownloadingList.vue";
import QrCodeViewer from "./DownloadComponents/QrCodeViewer.vue";
import {useMessage, useNotification} from "naive-ui";

const notification = useNotification();
const message = useMessage();

const config = defineModel<Config | undefined>("config", {required: true});

const biliCookie = ref<string>(config.value?.biliCookie ?? "");
const qrCodeViewerShowing = ref<boolean>(false);
const searchData = ref<SearchData>();
const currentTabName = ref<"search" | "episode">("search");
const mangaId = ref<number>();
const episodes = ref<Episode[] | undefined>();

watch(biliCookie, (value) => {
  if (config.value === undefined) {
    return;
  }
  config.value.biliCookie = value;
});

async function checkBiliCookie() {
  let result = await commands.getBiliCookieStatusData(biliCookie.value);
  if (result.status === "error") {
    notification.error({title: "检测失败", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "检测失败", description: response.msg});
    return;
  }
  let {isLogin} = response.data;
  if (isLogin) {
    message.success("Cookie有效");
  } else {
    message.error("Cookie无效");
  }
}

async function test() {
  notification.warning({
    title: "titletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitletitle",
    content: "contentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontentcontent",
    description: "descriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescriptiondescription",
    meta: "metametametametametametametametametametametametametametametametametametametametametametametametametametametametametametametametametameta "
  });
}


</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex">
      <n-input v-model:value="biliCookie" placeholder="" clearable>
        <template #prefix>
          Cookie:
        </template>
      </n-input>
      <n-button @click="qrCodeViewerShowing=true">二维码登录</n-button>
      <n-button @click="checkBiliCookie">检测</n-button>
      <n-button @click="test">测试用</n-button>
    </div>
    <div class="flex flex-1 overflow-hidden">
      <div class="basis-1/2 overflow-auto">
        <n-tabs v-model:value="currentTabName" type="line" size="small" class="h-full">
          <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show:lazy">
            <search-pane v-model:search-data="searchData"
                         v-model:manga-id="mangaId"
                         v-model:episodes="episodes"
                         v-model:current-tab-name="currentTabName"/>
          </n-tab-pane>
          <n-tab-pane class="h-full overflow-auto p-0!" name="episode" tab="章节详情" display-directive="show:lazy">
            <episode-pane v-model:manga-id="mangaId" v-model:episodes="episodes"/>
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
