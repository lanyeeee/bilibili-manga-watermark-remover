<script setup lang="ts">
import {MangaDirData} from "../bindings.ts";
import {showPathInFileManager} from "../utils.ts";

defineProps<{
  mangaDirExist: boolean,
  outputDirExist: boolean,
  imagesExist: boolean,
  mangaDirDataList: MangaDirData[],
  loadBackground: () => Promise<void>,
}>();

const cropperShowing = defineModel<boolean>("cropperShowing", {required: true});
const cropperWidth = defineModel<number>("cropperWidth", {required: true});
const cropperHeight = defineModel<number>("cropperHeight", {required: true});

function showCropper(width: number, height: number) {
  cropperShowing.value = true;
  cropperWidth.value = width;
  cropperHeight.value = height;
}

</script>

<template>
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
</template>