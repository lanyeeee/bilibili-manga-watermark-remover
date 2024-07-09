<script setup lang="ts">
import {onMounted, ref} from "vue";
import {open} from "@tauri-apps/plugin-dialog";
import {commands, ImageSizeCount, JpgImageData} from "./bindings.ts";
import WatermarkCropper from "./components/WatermarkCropper.vue";

const mangaDir = ref<string>();
const outputDir = ref<string>();
const imageSizeCounts = ref<ImageSizeCount[]>([]);
const blackBackground = ref<JpgImageData>();
const whiteBackground = ref<JpgImageData>();

onMounted(async () => {
  outputDir.value = await commands.getUserDownloadPath() || undefined;
  await loadBackground();
});


async function removeWatermark() {
  if (mangaDir.value === undefined) {
    console.error("请选择漫画文件夹");
    return;
  }
  if (imageSizeCounts.value.length === 0) {
    console.error("没有图片尺寸统计信息");
    return;
  }
  if (outputDir.value === undefined) {
    console.error("请选择输出文件夹");
    return;
  }
  if (blackBackground.value === undefined) {
    console.error("缺少黑色背景图");
    return;
  }
  if (whiteBackground.value === undefined) {
    console.error("缺少白色背景图");
    return;
  }
  const [blackHeight, blackWidth] = [blackBackground.value.info.height, blackBackground.value.info.width];
  const [whiteHeight, whiteWidth] = [whiteBackground.value.info.height, whiteBackground.value.info.width];
  const [height, width] = [imageSizeCounts.value[0].height, imageSizeCounts.value[0].width];
  if (blackHeight !== height || blackWidth !== width) {
    console.error("漫画图片尺寸和黑色背景图尺寸不一致");
    return;
  }
  if (whiteHeight !== height || whiteWidth !== width) {
    console.error("漫画图片尺寸和白色背景图尺寸不一致");
    return;
  }

  let result = await commands.removeWatermark(mangaDir.value, outputDir.value, blackBackground.value, whiteBackground.value);
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
  imageSizeCounts.value = await commands.getImageSizeCount(selectedDirPath);
  mangaDir.value = selectedDirPath;
}

async function selectOutputDir() {
  const dirPath = await open({directory: true, defaultPath: outputDir.value});
  if (dirPath === null) {
    return;
  }
  outputDir.value = dirPath;
}

async function showPathInFileManager(path: string | undefined) {
  if (path === undefined) {
    return;
  }
  await commands.showPathInFileManager(path);
}

async function loadBackground() {
  const load = async (isBlack: boolean) => {
    const existsResult = await commands.backgroundExists(isBlack);
    if (existsResult.status === "error") {
      console.error(existsResult.error);
      return;
    }
    const exists = existsResult.data;
    if (!exists) {
      return;
    }
    const openResult = await commands.openBackground(isBlack);
    if (openResult.status === "error") {
      console.error(openResult.error);
      return;
    }
    const background = isBlack ? blackBackground : whiteBackground;
    background.value = openResult.data;
  };

  await Promise.all([load(true), load(false)]);
}

async function test() {
  console.log(outputDir.value);
}

</script>

<template>
  <n-modal-provider>
    <div class="flex flex-col">
      <n-button @click="selectMangaDir">1.选择漫画文件夹</n-button>
      <div v-for="size in imageSizeCounts" :key="size.count">
        <span>{{ size.height }}x{{ size.width }}: {{ size.count }}</span>
      </div>
      <watermark-cropper v-show="mangaDir!==undefined"
                         :manga-dir="mangaDir"
                         :image-size-counts="imageSizeCounts"
                         v-model:black-background="blackBackground"
                         v-model:white-background="whiteBackground"
      />
      <n-button :disabled="blackBackground===undefined"
                @click="showPathInFileManager(blackBackground?.info.path)"
      >
        查看黑色背景
      </n-button>
      <n-button :disabled="whiteBackground===undefined"
                @click="showPathInFileManager(whiteBackground?.info.path)"
      >
        查看白色背景
      </n-button>
      <n-button @click="selectOutputDir">4.选择输出文件夹</n-button>
      <n-button @click="removeWatermark">5.开始去水印</n-button>
      <n-button @click="test">测试</n-button>
    </div>
  </n-modal-provider>
</template>
