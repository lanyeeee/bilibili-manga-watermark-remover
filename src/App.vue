<script setup lang="ts">
import {onMounted, ref} from "vue";
import {open} from "@tauri-apps/plugin-dialog";
import BackgroundCropper from "./components/BackgroundCropper.vue";
import {commands, JpgImage, MangaSize} from "./bindings.ts";

const mangaDir = ref<string>();
const outputDir = ref<string>();
const showCropper = ref<boolean>(false);
const isBlackCropper = ref<boolean>(false);
const blackImage = ref<JpgImage | null>(null);
const whiteImage = ref<JpgImage | null>(null);
const mangaSizes = ref<MangaSize[]>([]);

onMounted(async () => {
  blackImage.value = await commands.openBackground(true);
  whiteImage.value = await commands.openBackground(false);
});


async function removeWatermark() {
  const blackExist = await commands.backgroundExists(true);
  const whiteExist = await commands.backgroundExists(false);
  if (!blackExist) {
    console.error("请选择黑色背景图");
    return;
  }
  if (!whiteExist) {
    console.error("请选择白色背景图");
    return;
  }
  if (mangaDir.value === undefined) {
    console.error("请选择漫画文件夹");
    return;
  }
  if (outputDir.value === undefined) {
    console.error("请选择输出文件夹");
    return;
  }

  await commands.removeWatermark(mangaDir.value, outputDir.value);
}

async function selectMangaDir() {
  const dirPath = await open({directory: true});
  if (dirPath === null) {
    return;
  }

  mangaDir.value = dirPath;
  mangaSizes.value = await commands.getMangaSizes(dirPath);
}

async function selectOutputDir() {
  const dirPath = await open({directory: true});
  if (dirPath === null) {
    return;
  }

  outputDir.value = dirPath;
}

function showBlackCropper() {
  isBlackCropper.value = true;
  showCropper.value = true;
}

function showWhiteCropper() {
  isBlackCropper.value = false;
  showCropper.value = true;
}

async function test() {
}

</script>

<template>
  <n-modal-provider>
    <div class="flex flex-col">
      <n-button @click="selectMangaDir">1.选择漫画文件夹</n-button>
      <div v-for="size in mangaSizes" :key="size.count">
        <span>{{ size.height }}x{{ size.width }}: {{ size.count }}</span>
      </div>
      <n-button @click="showBlackCropper">2.框出黑色背景的水印</n-button>
      <n-button @click="showWhiteCropper">3.框出白色背景的水印</n-button>
      <n-button @click="selectOutputDir">4.选择输出文件夹</n-button>
      <n-button @click="removeWatermark">5.开始去水印</n-button>
      <n-button @click="test">测试</n-button>
    </div>
    <n-modal v-model:show="showCropper" :mask-closable="false">
      <background-cropper :is-black="isBlackCropper"
                          :manga-dir="mangaDir"
                          v-model:show="showCropper"
                          v-model:black-image="blackImage"
                          v-model:white-image="whiteImage"
      />
    </n-modal>
  </n-modal-provider>
</template>
