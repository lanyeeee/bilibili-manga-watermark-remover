<script setup lang="ts">
import {SearchData} from "../../bindings.ts";

defineProps<{
  searchData: SearchData | undefined;
  onClickItem: (id: number) => void;
}>();

</script>

<template>
  <div class="flex flex-col gap-row-2 overflow-auto p-2">
    <n-card class="cursor-pointer"
            content-style="padding: 0.25rem;"
            hoverable
            v-for="item in searchData?.list"
            :key="item.id"
            @click="onClickItem(item.id)">
      <div class="flex">
        <img class="w-32 object-cover pr-4"
             :src="`${item.vertical_cover}@310w.jpg`"
             alt=""
             referrerpolicy="no-referrer"/>
        <div class="flex flex-col h-full">
          <span v-html="item.title" class="font-bold text-xl"></span>
          <span v-html="item.author_name"></span>
          <span v-html="item.styles"></span>
          <span>{{ item.is_finish ? "已完结" : "连载中" }}</span>
        </div>
      </div>
    </n-card>
  </div>
</template>

<style scoped>
:deep(.keyword) {
  @apply not-italic text-blue-400
}
</style>