<script setup lang="ts">
import {computed, onMounted, ref} from "vue";
import {useMessage, useNotification} from "naive-ui";
import {open} from "@tauri-apps/plugin-dialog";
import {commands, events, JpgImageData, MangaDirData} from "./bindings.ts";
import WatermarkCropper from "./components/WatermarkCropper.vue";
import StatusIndicator from "./components/StatusIndicator.vue";
import {path} from "@tauri-apps/api";
import {BaseDirectory, exists} from "@tauri-apps/plugin-fs";
import RemoveProgress from "./components/RemoveProgress.vue";

const notification = useNotification();
const message = useMessage();

const mangaDir = ref<string>();
const outputDir = ref<string>();
const mangaDirDataList = ref<MangaDirData[]>([]);
const removeWatermarkTasks = ref<Map<string, [number, number]>>(new Map());

const cropperShowing = ref<boolean>(false);
const cropperWidth = ref<number>(0);
const cropperHeight = ref<number>(0);

const mangaDirExist = computed<boolean>(() => mangaDir.value !== undefined);
const outputDirExist = computed<boolean>(() => outputDir.value !== undefined);
const imagesExist = computed<boolean>(() => mangaDirDataList.value.length > 0);
const removeWatermarkButtonDisabled = computed<boolean>(() => !mangaDirExist.value || !outputDirExist.value);


onMounted(async () => {
  await events.removeWatermarkStartEvent.listen((event) => {
    const {dir_path, total} = event.payload;
    removeWatermarkTasks.value.set(dir_path, [0, total]);
  });
  await events.removeWatermarkSuccessEvent.listen((event) => {
    const {dir_path, current} = event.payload;
    const entry = removeWatermarkTasks.value.get(dir_path) as [number, number] | undefined;
    if (entry === undefined) {
      return;
    }
    entry[0] = current;
  });
  await events.removeWatermarkEndEvent.listen((event) => {
    const {dir_path} = event.payload;
    removeWatermarkTasks.value.delete(dir_path);
  });

  outputDir.value = await path.resourceDir();
  await loadBackground();
});

async function removeWatermark() {
  if (mangaDir.value === undefined) {
    message.error("请选择漫画文件夹");
    return;
  }
  if (mangaDirDataList.value.length === 0) {
    message.error("没有图片尺寸统计信息");
    return;
  }
  if (outputDir.value === undefined) {
    message.error("请选择输出文件夹");
    return;
  }

  const backgrounds_data: [JpgImageData, JpgImageData][] = mangaDirDataList.value
      .filter(data => data.blackBackground !== null && data.whiteBackground !== null)
      .map(data => [data.blackBackground as JpgImageData, data.whiteBackground as JpgImageData]);
  let result = await commands.removeWatermark(mangaDir.value, outputDir.value, backgrounds_data);
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
  const result = await commands.getMangaDirData(selectedDirPath);
  if (result.status === "error") {
    notification.error({title: "获取漫画目录数据", description: result.error});
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({title: "获取漫画目录数据", description: response.msg});
    return;
  }
  mangaDirDataList.value = response.data;
  mangaDir.value = selectedDirPath;

  const generatingMessage = message.loading("尝试自动生成背景水印图", {duration: 0});
  for (const mangaDirData of mangaDirDataList.value) {
    if (mangaDirData.blackBackground !== null && mangaDirData.whiteBackground !== null) {
      message.info(`尺寸(${mangaDirData.width}x${mangaDirData.height})的背景水印图已存在，跳过自动生成`);
      continue;
    }
    const generateResult = await commands.generateBackground(mangaDir.value, null, mangaDirData.width, mangaDirData.height);
    if (generateResult.status === "error") {
      notification.error({
        title: `自动生成背景水印图(${mangaDirData.width}x${mangaDirData.height})失败`,
        description: generateResult.error
      });
      continue;
    }
    const response = generateResult.data;
    if (response.code !== 0) {
      notification.warning({
        title: `自动生成背景水印图(${mangaDirData.width}x${mangaDirData.height})失败`,
        description: response.msg,
        content: "请尝试手动截取水印",
      });
      continue;
    }
    message.success(`自动生成背景水印图(${mangaDirData.width}x${mangaDirData.height})成功`);
  }
  generatingMessage.destroy();
  await loadBackground();
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
  console.log(path);
  await commands.showPathInFileManager(path);
}

async function loadBackground() {
  const tasks = [];
  for (const mangaDirData of mangaDirDataList.value) {
    const load = async (isBlack: boolean) => {
      const filename = isBlack ? "black.png" : "white.png";
      const backgroundRelativePath = await path.join(`背景水印图/${mangaDirData.width}x${mangaDirData.height}`, filename);
      const backgroundExist = await exists(backgroundRelativePath, {baseDir: BaseDirectory.Resource});
      if (!backgroundExist) {
        return;
      }
      const resourceDir = await path.resourceDir();
      const backgroundPath = await path.join(resourceDir, backgroundRelativePath);
      const result = await commands.openImage(backgroundPath);
      if (result.status === "error") {
        notification.error({title: "打开背景水印图失败", description: result.error});
        return;
      }
      const response = result.data;
      if (response.code !== 0) {
        notification.warning({title: "打开背景水印图失败", description: response.msg});
        return;
      }
      if (isBlack) {
        mangaDirData.blackBackground = response.data;
      } else {
        mangaDirData.whiteBackground = response.data;
      }
    };
    mangaDirData.blackBackground = null;
    mangaDirData.whiteBackground = null;
    tasks.push(load(true), load(false));
  }
  await Promise.all(tasks);
}

function showCropper(width: number, height: number) {
  cropperShowing.value = true;
  cropperWidth.value = width;
  cropperHeight.value = height;
}

async function test() {
  console.log(mangaDirDataList.value);
}

</script>

<template>
  <div class="flex flex-col">
    <status-indicator content="选择漫画目录" :ok="mangaDirExist"/>
    <status-indicator content="选择输出目录" :ok="outputDirExist"/>
    <status-indicator v-if="mangaDirExist" content="漫画目录存在图片" :ok="imagesExist"/>

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
      <div v-else v-for="dirData in mangaDirDataList" :key="dirData.count">
        <span>
          尺寸({{ dirData.width }}x{{ dirData.height }})共有{{ dirData.count }} 张
          <n-button size="tiny"
                    :disabled="dirData.blackBackground===null"
                    @click="showPathInFileManager(dirData.blackBackground?.info.path)">
            黑色
          </n-button>
          <n-button size="tiny"
                    :disabled="dirData.whiteBackground===null"
                    @click="showPathInFileManager(dirData.whiteBackground?.info.path)">
            白色
          </n-button>
          <n-button size="tiny" @click="showCropper(dirData.width, dirData.height)">手动截取水印</n-button>
          <span v-if="dirData.blackBackground!==null&&dirData.whiteBackground!==null">✅将被去除水印</span>
          <span v-else-if="dirData.blackBackground===null&&dirData.whiteBackground===null">
            ❌将被复制，因为缺少黑色和白色背景水印图
          </span>
          <span v-else-if="dirData.blackBackground===null">❌将被复制，因为缺少黑色背景水印图</span>
          <span v-else-if="dirData.whiteBackground===null">❌将被复制，因为缺少白色背景水印图</span>
        </span>
      </div>
    </div>

    <n-button :disabled="removeWatermarkButtonDisabled"
              type="primary"
              @click="removeWatermark">
      开始去水印
    </n-button>
    <n-button @click="test">测试用</n-button>

    <RemoveProgress :remove-watermark-tasks="removeWatermarkTasks"/>
  </div>
  <n-modal v-model:show="cropperShowing">
    <watermark-cropper :manga-dir="mangaDir"
                       :manga-dir-data-list="mangaDirDataList"
                       :load-background="loadBackground"
                       :width="cropperWidth"
                       :height="cropperHeight"
                       v-model:showing="cropperShowing"/>
  </n-modal>

</template>
