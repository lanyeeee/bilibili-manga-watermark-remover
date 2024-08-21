<script setup lang="ts">
import {SelectionArea, SelectionEvent, SelectionOptions} from "@viselect/vue";
import {nextTick, ref, watch} from "vue";
import {MangaData} from "../../bindings.ts";

const mangaData = defineModel<MangaData | undefined>("mangaData", {required: true});

const dropdownX = ref(0);
const dropdownY = ref(0);
const showDropdown = ref(false);
const dropdownOptions = [
  {label: "勾选", key: "check"},
  {label: "取消勾选", key: "uncheck"},
  {label: "全选", key: "check all"},
  {label: "取消全选", key: "uncheck all"},
];
const checkedIds = ref<number[]>([]);
const selectedIds = ref<Set<number>>(new Set());
// 创建一个变量，记录这次框选是否改动了选中的元素
const selectedChanged = ref(false);

watch(selectedIds.value, () => {
  selectedChanged.value = true;
});

function extractIds(elements: Element[]): number[] {
  return elements.map(element => element.getAttribute("data-key"))
      .filter(Boolean)
      .map(Number)
      .filter(epIsUnlocked);
}

function onMouseDown(event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    return;
  }
  if (event?.button === 0) {
    selectedChanged.value = false;
  }
}

function onMouseUp(event: MouseEvent) {
  // 如果是左键点击，且没有改动选中的元素，则清空选中
  if (event?.button === 0 && !selectedChanged.value) {
    selectedIds.value.clear();
  }
}

function onDragStart({event, selection}: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection();
    selectedIds.value.clear();
  }
}

function onDragMove({store: {changed: {added, removed}}}: SelectionEvent) {
  extractIds(added).forEach(id => selectedIds.value.add(id));
  extractIds(removed).forEach(id => selectedIds.value.delete(id));
}

function onDropdownSelect(key: "check" | "uncheck" | "check all" | "uncheck all") {
  showDropdown.value = false;
  if (key === "check") {
    // 只有未勾选的才会被勾选
    [...selectedIds.value]
        .filter(id => !checkedIds.value.includes(id))
        .forEach(id => checkedIds.value.push(id));
  } else if (key === "uncheck") {
    checkedIds.value = checkedIds.value.filter(id => !selectedIds.value.has(id));
  } else if (key === "check all") {
    // 只有未锁定的才会被勾选
    mangaData.value?.ep_list
        .filter(ep => !ep.is_locked)
        .forEach(ep => checkedIds.value.push(ep.id));
  } else if (key === "uncheck all") {
    checkedIds.value.length = 0;
  }
}

async function onContextMenu(e: MouseEvent) {
  showDropdown.value = false;
  await nextTick();
  showDropdown.value = true;
  dropdownX.value = e.clientX;
  dropdownY.value = e.clientY;
}

function epIsUnlocked(id: number): boolean {
  return !mangaData.value?.ep_list.find(ep => ep.id === id)?.is_locked ?? false;
}

function test() {
  console.log(checkedIds.value);
  console.log(selectedIds.value);
}
</script>

<template>
  <div class="h-full flex flex-col">
    <n-button @click="test">测试用</n-button>
    <div class="flex flex-justify-around">
      <span>总章数：{{ mangaData?.ep_list.length }}</span>
      <n-divider vertical></n-divider>
      <span>已解锁：{{ mangaData?.ep_list.filter(ep => !ep.is_locked).length }}</span>
      <n-divider vertical></n-divider>
      <span>已下载：待完善</span>
      <n-divider vertical></n-divider>
      <span>已选中：{{ checkedIds.length }}</span>
    </div>
    <SelectionArea class="selection-container"
                   :options="{selectables: '.selectable'} as SelectionOptions"
                   @contextmenu="onContextMenu"
                   @mousedown="onMouseDown"
                   @mouseup="onMouseUp"
                   @move="onDragMove"
                   @start="onDragStart">
      <n-checkbox-group v-model:value="checkedIds" class="grid grid-cols-3 gap-3 w-full">
        <n-checkbox v-for="{id, title, is_locked} of mangaData?.ep_list"
                    :key="id"
                    :data-key="id"
                    class="selectable hover:bg-gray-200!"
                    :value="id"
                    :label="title"
                    :disabled="is_locked"
                    :class="{ selected: selectedIds.has(id) }"/>
      </n-checkbox-group>
    </SelectionArea>

    <n-dropdown
        placement="bottom-start"
        trigger="manual"
        :x="dropdownX"
        :y="dropdownY"
        :options="dropdownOptions"
        :show="showDropdown"
        :on-clickoutside="()=>showDropdown=false"
        @select="onDropdownSelect"
    />
  </div>
</template>

<style scoped>
.selection-container {
  display: -webkit-box;
  display: -ms-flexbox;
  user-select: none;
  overflow: auto;
}

.selection-container .selected {
  background: rgba(24, 160, 88, 0.16);
}


</style>

<style>
.selection-area {
  background: rgba(46, 115, 252, 0.5);
  border: 1px solid rgba(98, 155, 255, 0.85);
  border-radius: 0.15em;
}
</style>
