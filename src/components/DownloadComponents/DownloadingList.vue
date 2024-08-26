<script setup lang="ts">

import {onMounted, ref} from "vue";
import {events} from "../../bindings.ts";
import {useNotification} from "naive-ui";

type ProgressData = {
  title: string;
  current: number;
  total: number;
  percentage: number;
}

const notification = useNotification();

const progresses = ref<Map<number, ProgressData>>(new Map());

onMounted(async () => {
  await events.downloadEpisodePendingEvent.listen(({payload}) => {
    let progressData: ProgressData = {
      title: `${payload.epId}等待中`,
      current: 0,
      total: 0,
      percentage: 0,
    };
    progresses.value.set(payload.epId, progressData);
  });

  await events.downloadEpisodeStartEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.epId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    progressData.total = payload.total;
    progressData.title = payload.title;
  });

  await events.downloadImageSuccessEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.epId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    progressData.current += 1;
    progressData.percentage = Math.round(progressData.current / progressData.total * 100);
  });

  await events.downloadImageErrorEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.epId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    notification.warning({
      title: "下载图片失败",
      description: payload.url,
      content: payload.errMsg,
      meta: progressData.title
    });
  });

  await events.downloadEpisodeEndEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.epId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    if (payload.errMsg !== null) {
      notification.warning({title: "下载章节失败", content: payload.errMsg, meta: progressData.title});
    }
    progresses.value.delete(payload.epId);
  });
});
</script>

<template>
  <div>
    <n-h3>下载列表</n-h3>
    <div class="flex flex-col gap-row-1">
      <div class="grid grid-cols-[2fr_4fr_1fr]" v-for="[epId, {title, current, total, percentage}] in progresses"
           :key="epId">
        <span class="text-ellipsis whitespace-nowrap overflow-hidden">{{ title }}</span>
        <n-progress :percentage="percentage"
                    indicator-placement="inside">
        </n-progress>
        <span>{{ current }}/{{ total }}</span>
      </div>
    </div>

  </div>
</template>