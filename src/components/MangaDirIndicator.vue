<script setup lang="ts">
import { MangaDirData } from '../bindings.ts'
import { autoGenerateBackground, getBackgroundDirAbsPath, showPathInFileManager } from '../utils.ts'
import { useMessage, useNotification } from 'naive-ui'
import { nextTick } from 'vue'

const notification = useNotification()
const message = useMessage()

const prop = defineProps<{
  mangaDir: string | undefined
  mangaDirExist: boolean
  imagesExist: boolean
  mangaDirDataList: MangaDirData[]
  loadBackground: () => Promise<void>
  autoGenerateAll: () => Promise<void>
}>()

const cropperShowing = defineModel<boolean>('cropperShowing', { required: true })
const cropperWidth = defineModel<number>('cropperWidth', { required: true })
const cropperHeight = defineModel<number>('cropperHeight', { required: true })

function showCropper(width: number, height: number) {
  cropperShowing.value = true
  cropperWidth.value = width
  cropperHeight.value = height
}

async function showBackgroundDirInFileManager(mangaDirData: MangaDirData) {
  if (prop.mangaDir === undefined) {
    return
  }
  const backgroundDirAbsPath = await getBackgroundDirAbsPath(
    prop.mangaDir,
    mangaDirData.width,
    mangaDirData.height,
    notification,
  )
  if (backgroundDirAbsPath === null) {
    return
  }
  await showPathInFileManager(backgroundDirAbsPath)
}

async function autoGenerateSingle(width: number, height: number) {
  if (prop.mangaDir === undefined) {
    return
  }
  const autoGeneratingMessage = message.loading(`尝试自动生成背景水印图(${width}x${height})`, { duration: 0 })
  const success = await autoGenerateBackground(prop.mangaDir, width, height, notification)
  if (success) {
    message.success(`自动生成背景水印图(${width}x${height})成功`)
  }
  await prop.loadBackground()
  await nextTick(autoGeneratingMessage.destroy)
}
</script>

<template>
  <div v-if="mangaDirExist">
    <div class="flex gap-col-3">
      <div>漫画目录的图片:</div>
      <n-button size="tiny" type="primary" secondary @click="loadBackground">重新扫描水印目录</n-button>
      <n-button size="tiny" type="primary" secondary @click="autoGenerateAll">全部重试自动生成</n-button>
    </div>
    <div v-if="!imagesExist">
      <span>没有图片</span>
    </div>
    <div v-else v-for="dirData in mangaDirDataList" :key="dirData.count">
      <span class="flex flex-justify-b">
        尺寸({{ dirData.width }}x{{ dirData.height }})共有{{ dirData.count }} 张
        <n-button size="tiny" @click="showBackgroundDirInFileManager(dirData)">水印目录</n-button>
        <n-button size="tiny" @click="autoGenerateSingle(dirData.width, dirData.height)">尝试自动生成</n-button>
        <n-button size="tiny" @click="showCropper(dirData.width, dirData.height)">手动截取水印</n-button>
        <span v-if="dirData.blackBackground !== null && dirData.whiteBackground !== null">✅将被去除水印</span>
        <span v-else-if="dirData.blackBackground === null && dirData.whiteBackground === null">
          ❌将被复制，因为缺少2张背景水印图
        </span>
        <span v-else>❌将被复制，因为只有1张背景水印图</span>
      </span>
    </div>
  </div>
</template>
