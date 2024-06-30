<script setup lang="ts">
import BackgroundMaker from "./components/BackgroundMaker.vue";
import {invoke} from "@tauri-apps/api/core";
import {ref} from "vue";
import {open} from "@tauri-apps/plugin-dialog";

const whitePath = ref<string>();
const blackPath = ref<string>();
const mangaDir = ref<string>();
const outputDir = ref<string>();


async function removeWatermark() {
  if (whitePath.value === undefined) {
    console.error("请选择白底水印图片")
    return;
  }
  if (blackPath.value === undefined) {
    console.error("请选择黑底水印图片")
    return;
  }
  if (mangaDir.value === undefined) {
    console.error("请选择漫画文件夹")
    return;
  }
  if (outputDir.value === undefined) {
    console.error("请选择输出文件夹")
    return;
  }

  await invoke("remove_watermark", {
    whitePath: whitePath.value,
    blackPath: blackPath.value,
    mangaDir: mangaDir.value,
    outputDir: outputDir.value,
  });
}

async function selectWhitePath() {
  const fileResponse = await open();
  if (fileResponse === null) {
    return;
  }

  whitePath.value = fileResponse.path;
}

async function selectBlackPath() {
  const fileResponse = await open();
  if (fileResponse === null) {
    return;
  }

  blackPath.value = fileResponse.path;
}

async function selectMangaDir() {
  const dirPath = await open({directory: true});
  if (dirPath === null) {
    return;
  }

  mangaDir.value = dirPath;
}

async function selectOutputDir() {
  const dirPath = await open({directory: true});
  if (dirPath === null) {
    return;
  }

  outputDir.value = dirPath;
}


</script>

<template>
  <div>
    <background-maker :is-black="true"/>
    <background-maker :is-black="false"/>
  </div>
  <button @click="selectWhitePath">选择白底水印图片</button>
  <button @click="selectBlackPath">选择黑底水印图片</button>
  <button @click="selectMangaDir">选择漫画文件夹</button>
  <button @click="selectOutputDir">选择输出文件夹</button>
  <button @click="removeWatermark">去水印</button>
  <n-button class="w-full">测试</n-button>
</template>
