<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {useMessage, useNotification} from "naive-ui";
import {open} from "@tauri-apps/plugin-dialog";
import {commands, ImageSizeCount, JpgImageData} from "./bindings.ts";
import WatermarkCropper from "./components/WatermarkCropper.vue";
import StatusIndicator from "./components/StatusIndicator.vue";
import {loadBackground} from "./utils.ts";
import {path} from "@tauri-apps/api";

const notification = useNotification();
const message = useMessage();

const mangaDir = ref<string>();
const outputDir = ref<string>();
const imageSizeCounts = ref<ImageSizeCount[]>([]);
const blackBackground = ref<JpgImageData>();
const whiteBackground = ref<JpgImageData>();
const showCropper = ref<boolean>(false);

const mangaDirExist = computed<boolean>(() => mangaDir.value !== undefined);
const outputDirExist = computed<boolean>(() => outputDir.value !== undefined);
const blackBackgroundExist = computed<boolean>(() => blackBackground.value !== undefined);
const whiteBackgroundExist = computed<boolean>(() => whiteBackground.value !== undefined);
const backgroundMatchManga = computed<boolean>(() => {
  if (imageSizeCounts.value.length === 0) {
    return false;
  }
  const [blackHeight, blackWidth] = [blackBackground.value?.info.height, blackBackground.value?.info.width];
  const [whiteHeight, whiteWidth] = [whiteBackground.value?.info.height, whiteBackground.value?.info.width];
  const [height, width] = [imageSizeCounts.value[0]?.height, imageSizeCounts.value[0]?.width];
  return blackHeight === height && blackWidth === width && whiteHeight === height && whiteWidth === width;
});
const imagesExist = computed<boolean>(() => imageSizeCounts.value.length > 0);
const removeWatermarkButtonDisabled = computed<boolean>(() =>
    !mangaDirExist.value ||
    !outputDirExist.value ||
    !blackBackgroundExist.value ||
    !whiteBackgroundExist.value ||
    !backgroundMatchManga.value
);

onMounted(async () => {
  outputDir.value = await path.resourceDir();
  await loadBackground(blackBackground, whiteBackground);
});

async function removeWatermark() {
  if (mangaDir.value === undefined) {
    message.error("请选择漫画文件夹");
    return;
  }
  if (imageSizeCounts.value.length === 0) {
    message.error("没有图片尺寸统计信息");
    return;
  }
  if (outputDir.value === undefined) {
    message.error("请选择输出文件夹");
    return;
  }
  if (blackBackground.value === undefined) {
    message.error("缺少黑色背景水印图");
    return;
  }
  if (whiteBackground.value === undefined) {
    message.error("缺少白色背景水印图");
    return;
  }

  let result = await commands.removeWatermark(mangaDir.value, outputDir.value, blackBackground.value, whiteBackground.value);
  if (result.status === "ok") {
    message.success("去水印成功");
  } else {
    notification.error({title: "去水印失败", content: result.error});
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
  if (!blackBackgroundExist.value || !whiteBackgroundExist.value || !backgroundMatchManga.value) {
    // TODO: 尝试自动生成背景水印图
  }
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

async function test() {
}

// TODO: 展示去水印进度
</script>

<template>
  <div class="flex flex-col">
    <status-indicator content="选择漫画目录" :ok="mangaDirExist"/>
    <status-indicator content="选择输出目录" :ok="outputDirExist"/>
    <status-indicator content="存在黑色背景水印图" :ok="blackBackgroundExist"/>
    <status-indicator content="存在白色背景水印图" :ok="whiteBackgroundExist"/>
    <status-indicator v-if="mangaDirExist" content="漫画目录存在图片" :ok="imagesExist"/>
    <status-indicator v-if="mangaDirExist && imagesExist"
                      content="水印图尺寸与漫画尺寸匹配"
                      :ok="backgroundMatchManga"/>

    <div class="flex">
      <n-input v-model:value="mangaDir"
               readonly
               placeholder="请选择漫画目录"
               @click="selectMangaDir">
        <template #prefix>漫画目录：</template>
      </n-input>
      <n-button :disabled="!mangaDirExist" @click="showPathInFileManager(mangaDir)">浏览目录</n-button>
    </div>

    <div class="flex">
      <n-input v-model:value="outputDir"
               readonly
               placeholder="请选择漫画目录"
               @click="selectOutputDir">
        <template #prefix>输出目录：</template>
      </n-input>
      <n-button :disabled="!outputDirExist" @click="showPathInFileManager(outputDir)">浏览目录</n-button>
    </div>

    <div v-if="mangaDirExist">
      漫画目录的图片:
      <div v-if="!imagesExist">
        <span>没有图片</span>
      </div>
      <div v-else v-for="size in imageSizeCounts" :key="size.count">
        <span>(高:{{ size.height }}, 宽{{ size.width }}): {{ size.count }} 张</span>
      </div>
    </div>

    <n-button :disabled="!mangaDirExist || !imagesExist" @click="showCropper=true">手动截取水印</n-button>

    <n-button :disabled="removeWatermarkButtonDisabled"
              type="primary"
              @click="removeWatermark">
      开始去水印
    </n-button>

    <n-button :disabled="!blackBackgroundExist"
              @click="showPathInFileManager(blackBackground?.info.path)">
      浏览黑色背景水印图
    </n-button>
    <n-button :disabled="!whiteBackgroundExist"
              @click="showPathInFileManager(whiteBackground?.info.path)">
      浏览白色背景水印图
    </n-button>

    <n-button @click="test">测试用</n-button>
  </div>
  <n-modal v-model:show="showCropper">
    <watermark-cropper :manga-dir="mangaDir"
                       :image-size-counts="imageSizeCounts"
                       v-model:black-background="blackBackground"
                       v-model:white-background="whiteBackground"
                       v-model:showing="showCropper"/>
  </n-modal>

</template>
