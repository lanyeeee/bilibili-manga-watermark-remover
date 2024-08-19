<script setup lang="ts">
import {SelectionArea, SelectionEvent, SelectionOptions} from "@viselect/vue";
import {ref, watch} from "vue";


const selected = ref<Set<number>>(new Set());
// 创建一个变量，记录这次框选是否改动了选中的元素
const selectedChanged = ref(false);

watch(selected.value, () => {
  selectedChanged.value = true;
});

function extractIds(elements: Element[]): number[] {
  return elements.map(element => element.getAttribute("data-key"))
      .filter(Boolean)
      .map(Number);
}

function onMouseDown(event: MouseEvent): void {
  if (event?.button === 0) {
    selectedChanged.value = false;
  }
}

function onMouseUp(event: MouseEvent): void {
  if (event?.button === 0 && !selectedChanged.value) {
    selected.value.clear();
  }
}

function onStart({event, selection}: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection();
    selected.value.clear();
  }
}

function onMove({store: {changed: {added, removed}}}: SelectionEvent) {
  extractIds(added).forEach(id => selected.value.add(id));
  extractIds(removed).forEach(id => selected.value.delete(id));
}

function range(to: number, offset = 0): number[] {
  return new Array(to).fill(0).map((_, i) => offset + i);
}

function test() {
  selected.value.clear();
}


</script>

<template>
  <div class="h-full flex flex-col">
    <n-button @click="test">测试用</n-button>
    <SelectionArea ref="selectionAreaRef"
                   class="selection-container"
                   :options="{selectables: '.selectable'} as SelectionOptions"
                   @mousedown="onMouseDown"
                   @mouseup="onMouseUp"
                   @move="onMove"
                   @start="onStart">
      <div v-for="id of range(800, 0)" :key="id" :data-key="id" class="selectable"
           :class="{ selected: selected.has(id) }"/>
    </SelectionArea>
  </div>
</template>

<style scoped>
.selection-container {
  display: -webkit-box;
  display: -ms-flexbox;
  display: flex;
  flex-wrap: wrap;
  user-select: none;
  overflow: auto;
}

.selection-container div {
  height: 3em;
  width: 3em;
  margin: 0.2em;
  background: rgba(66, 68, 90, 0.075);
  border: 2px solid transparent;
  border-radius: 0.15em;
  cursor: pointer;
}

.selection-container div.selected {
  background: red;
  border: 2px solid rgba(0, 0, 0, 0.075);
}
</style>

<style>
.selection-area {
  background: rgba(46, 115, 252, 0.5);
  border: 1px solid rgba(98, 155, 255, 0.85);
  border-radius: 0.15em;
}
</style>
