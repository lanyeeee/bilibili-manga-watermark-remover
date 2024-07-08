<script setup lang="ts">
import {onMounted, ref} from "vue";
import {open} from "@tauri-apps/plugin-dialog";
import {commands, ImageSizeCount} from "./bindings.ts";
import WatermarkCropper from "./components/WatermarkCropper.vue";

const mangaDir = ref<string>();
const outputDir = ref<string>();
const imageSizeCounts = ref<ImageSizeCount[]>([]);

onMounted(async () => {
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

  let result = await commands.removeWatermark(mangaDir.value, outputDir.value);
  if (result.status === "ok") {
    console.log("去水印成功");
  } else {
    console.error(result.error);
  }
}

async function selectMangaDir() {
  const selectedDirPath = await open({directory: true});
  if (selectedDirPath === null) {
    return;
  }
  // 获取图片尺寸统计
  console.log(mangaDir.value);
  imageSizeCounts.value = await commands.getImageSizeCount(selectedDirPath);
  mangaDir.value = selectedDirPath;
  console.log(mangaDir.value);
}

async function selectOutputDir() {
  const dirPath = await open({directory: true});
  if (dirPath === null) {
    return;
  }

  outputDir.value = dirPath;
}

async function test() {
  console.log(mangaDir.value)
}

</script>

<template>
  <n-modal-provider>
    <div class="flex flex-col">
      <n-button @click="selectMangaDir">1.选择漫画文件夹</n-button>
      <div v-for="size in imageSizeCounts" :key="size.count">
        <span>{{ size.height }}x{{ size.width }}: {{ size.count }}</span>
      </div>
      <watermark-cropper :manga-dir="mangaDir" :image-size-counts="imageSizeCounts"/>
      <n-button @click="selectOutputDir">4.选择输出文件夹</n-button>
      <n-button @click="removeWatermark">5.开始去水印</n-button>
      <n-button @click="test">测试</n-button>
    </div>
  </n-modal-provider>
</template>
