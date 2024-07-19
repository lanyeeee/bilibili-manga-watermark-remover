<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {useMessage, useNotification} from "naive-ui";
import {open} from "@tauri-apps/plugin-dialog";
import {commands, events, ImageSizeCount, JpgImageData} from "./bindings.ts";
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
const removeWatermarkProgress = ref<Map<string, [number, number]>>(new Map());

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
  await events.removeWatermarkStartEvent.listen((event) => {
    const {dir_path, total} = event.payload;
    removeWatermarkProgress.value.set(dir_path, [0, total]);
  });

  await events.removeWatermarkSuccessEvent.listen((event) => {
    const {dir_path, current} = event.payload;
    const entry = removeWatermarkProgress.value.get(dir_path) as [number, number] | undefined;
    if (entry === undefined) {
      return;
    }
    entry[0] = current;
  });
  await events.removeWatermarkEndEvent.listen((event) => {
    const {dir_path} = event.payload;
    removeWatermarkProgress.value.delete(dir_path);
    message.success(`${dir_path} 去水印成功`);
  });

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
  if (result.status === "error") {
    notification.error({title: "去水印失败", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "去水印失败", description: response.msg});
    return;
  }
  message.success("去水印成功");
}

async function selectMangaDir() {
  const selectedDirPath = await open({directory: true});
  if (selectedDirPath === null) {
    return;
  }
  const response = await commands.getImageSizeCount(selectedDirPath);
  if (response.code !== 0) {
    notification.warning({title: "获取图片尺寸统计失败", description: response.msg});
    return;
  }
  imageSizeCounts.value = response.data;
  mangaDir.value = selectedDirPath;
  // 如果漫画目录下没有图片，则无法生成背景水印图
  if (imageSizeCounts.value.length === 0) {
    return;
  }
  // 如果黑色背景水印图和白色背景水印图都存在，且尺寸与漫画尺寸匹配，则无需生成背景水印图
  if (blackBackgroundExist.value && whiteBackgroundExist.value && backgroundMatchManga.value) {
    return;
  }
  // 否则尝试生成背景水印图
  const generatingMessage = message.loading("尝试自动生成背景水印图", {duration: 0});
  const height = imageSizeCounts.value[0].height;
  const width = imageSizeCounts.value[0].width;
  const generateResult = await commands.generateBackground(mangaDir.value, null, width, height);
  if (generateResult.status === "error") {
    generatingMessage.destroy();
    notification.error({
      title: "自动生成背景水印图失败",
      description: generateResult.error,
      content: "请尝试手动截取水印"
    });
    return;
  }
  generatingMessage.destroy();
  const generateResponse = generateResult.data;
  if (generateResponse.code !== 0) {
    notification.warning({title: "自动生成背景水印图失败", description: generateResponse.msg});
    return;
  }
  message.success("自动生成背景水印图成功");

  await loadBackground(blackBackground, whiteBackground);
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
  console.log(path)
  await commands.showPathInFileManager(path);
}

async function test() {

}

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
      <n-button :disabled="!mangaDirExist" @click="showPathInFileManager(mangaDir)">打开目录</n-button>
    </div>

    <div class="flex">
      <n-input v-model:value="outputDir"
               readonly
               placeholder="请选择漫画目录"
               @click="selectOutputDir">
        <template #prefix>输出目录：</template>
      </n-input>
      <n-button :disabled="!outputDirExist" @click="showPathInFileManager(outputDir)">打开目录</n-button>
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
      打开黑色背景水印图目录
    </n-button>
    <n-button :disabled="!whiteBackgroundExist"
              @click="showPathInFileManager(whiteBackground?.info.path)">
      打开白色背景水印图目录
    </n-button>

    <n-button @click="test">测试用</n-button>
    <div v-for="(progress, dirPath) in removeWatermarkProgress" :key="dirPath">
      <span>{{ dirPath }}: {{ progress[0] }} / {{ progress[1] }}</span>
    </div>
  </div>
  <n-modal v-model:show="showCropper">
    <watermark-cropper :manga-dir="mangaDir"
                       :image-size-counts="imageSizeCounts"
                       v-model:black-background="blackBackground"
                       v-model:white-background="whiteBackground"
                       v-model:showing="showCropper"/>
  </n-modal>

</template>
