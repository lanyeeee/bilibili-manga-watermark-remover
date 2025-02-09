<script setup lang="ts">
import { useMessage, useNotification } from 'naive-ui'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { commands, Config, events, JpgImageData, MangaDirData } from './bindings.ts'
import {
  autoGenerateBackground,
  getBackgroundDirAbsPath,
  getBackgroundDirRelativePath,
  showPathInFileManager,
} from './utils.ts'
import RemoveProgress from './components/RemoveProgress.vue'
import WatermarkCropper from './components/WatermarkCropper.vue'
import MangaDirIndicator from './components/MangaDirIndicator.vue'
import { open } from '@tauri-apps/plugin-dialog'
import { path } from '@tauri-apps/api'
import { BaseDirectory, exists } from '@tauri-apps/plugin-fs'

const message = useMessage()
const notification = useNotification()

const config = ref<Config>()

const mangaDir = ref<string>()
const mangaDirDataList = ref<MangaDirData[]>([])
const removeWatermarkTasks = ref<Map<string, [number, number]>>(new Map())

const cropperShowing = ref<boolean>(false)
const cropperWidth = ref<number>(0)
const cropperHeight = ref<number>(0)

const mangaDirExist = computed<boolean>(() => mangaDir.value !== undefined)
const imagesExist = computed<boolean>(() => mangaDirDataList.value.length > 0)
const removeWatermarkButtonDisabled = computed<boolean>(() => !mangaDirExist.value || !imagesExist.value)

watch(
  config,
  async () => {
    if (config.value === undefined) {
      return
    }

    await events.removeWatermarkStartEvent.listen((event) => {
      const { dirPath, total } = event.payload
      removeWatermarkTasks.value.set(dirPath, [0, total])
    })
    await events.removeWatermarkSuccessEvent.listen((event) => {
      const { dirPath, current } = event.payload
      const entry = removeWatermarkTasks.value.get(dirPath) as [number, number] | undefined
      if (entry === undefined) {
        return
      }
      entry[0] = current
    })
    await events.removeWatermarkEndEvent.listen((event) => {
      const { dirPath } = event.payload
      removeWatermarkTasks.value.delete(dirPath)
    })

    const result = await commands.saveConfig(config.value)
    if (result.status === 'error') {
      notification.error({ title: '保存配置失败', description: result.error })
      return
    }
    message.success('保存配置成功')
  },
  { deep: true },
)

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault()
  }
  // 获取配置
  config.value = await commands.getConfig()
})

async function removeWatermark() {
  if (config.value === undefined) {
    message.error('配置未加载')
    return
  }
  if (mangaDir.value === undefined) {
    message.error('请选择漫画文件夹')
    return
  }
  if (mangaDirDataList.value.length === 0) {
    message.error('没有图片尺寸统计信息')
    return
  }

  const backgroundsData: [JpgImageData, JpgImageData][] = mangaDirDataList.value
    .filter((data) => data.blackBackground !== null && data.whiteBackground !== null)
    .map((data) => [data.blackBackground as JpgImageData, data.whiteBackground as JpgImageData])
  const cfg = config.value
  let result = await commands.removeWatermark(
    mangaDir.value,
    cfg.outputDir,
    cfg.outputFormat,
    cfg.outputOptimize,
    backgroundsData,
  )
  if (result.status === 'error') {
    notification.error({ title: '去水印失败', description: result.error })
    return
  }
  message.success('去水印成功')
}

async function autoGenerateAll() {
  if (mangaDir.value === undefined) {
    message.error('请选择漫画目录')
    return
  }
  const generatingMessage = message.loading('尝试自动生成背景水印图', { duration: 0 })
  for (const mangaDirData of mangaDirDataList.value) {
    if (mangaDirData.blackBackground !== null && mangaDirData.whiteBackground !== null) {
      message.info(`尺寸(${mangaDirData.width}x${mangaDirData.height})的背景水印图已存在，跳过自动生成`)
      continue
    }
    const success = await autoGenerateBackground(mangaDir.value, mangaDirData.width, mangaDirData.height, notification)
    if (!success) {
      continue
    }
    message.success(`自动生成背景水印图(${mangaDirData.width}x${mangaDirData.height})成功`)
  }
  // 使用 nextTick 保证生成消息能够被销毁
  await nextTick(generatingMessage.destroy)
  await loadBackground()
}

async function selectMangaDir() {
  const selectedDirPath = await open({ directory: true })
  if (selectedDirPath === null) {
    return
  }
  const result = await commands.getMangaDirData(selectedDirPath)
  if (result.status === 'error') {
    notification.error({ title: '获取漫画目录数据', description: result.error })
    return
  }
  mangaDirDataList.value = result.data
  mangaDir.value = selectedDirPath
  await autoGenerateAll()
}

async function selectOutputDir() {
  if (config.value === undefined) {
    message.error('配置未加载')
    return
  }
  const dirPath = await open({ directory: true, defaultPath: config.value.outputDir })
  if (dirPath === null) {
    return
  }
  config.value.outputDir = dirPath
}

async function loadBackground() {
  const tasks: Promise<void>[] = []
  for (const mangaDirData of mangaDirDataList.value) {
    const load = async (isBlack: boolean) => {
      if (mangaDir.value === undefined) {
        return
      }
      const filename = isBlack ? 'black.png' : 'white.png'
      // 检查背景水印图是否存在
      const backgroundDirRelativePath = await getBackgroundDirRelativePath(
        mangaDir.value,
        mangaDirData.width,
        mangaDirData.height,
        notification,
      )
      if (backgroundDirRelativePath === null) {
        return
      }
      const backgroundRelativePath = await path.join(backgroundDirRelativePath, filename)
      const backgroundExist = await exists(backgroundRelativePath, { baseDir: BaseDirectory.Resource })
      if (!backgroundExist) {
        return
      }
      // 加载背景水印图
      const backgroundDirAbsPath = await getBackgroundDirAbsPath(
        mangaDir.value,
        mangaDirData.width,
        mangaDirData.height,
        notification,
      )
      if (backgroundDirAbsPath === null) {
        return
      }
      const backgroundAbsPath = await path.join(backgroundDirAbsPath, filename)
      const result = await commands.openImage(backgroundAbsPath)
      if (result.status === 'error') {
        notification.error({ title: '打开背景水印图失败', description: result.error })
        return
      }
      if (isBlack) {
        mangaDirData.blackBackground = result.data
      } else {
        mangaDirData.whiteBackground = result.data
      }
    }
    mangaDirData.blackBackground = null
    mangaDirData.whiteBackground = null
    tasks.push(load(true), load(false))
  }
  await Promise.all(tasks)
}

async function test() {
  const cfg = await commands.getConfig()
  console.log(cfg)
}
</script>

<template>
  <div class="flex flex-col">
    <span>{{ mangaDirExist ? '✅' : '❌' }}选择漫画目录</span>
    <span>{{ imagesExist ? '✅' : '❌' }}漫画目录存在图片</span>

    <div class="flex">
      <n-input v-model:value="mangaDir" readonly placeholder="请选择漫画目录" @click="selectMangaDir">
        <template #prefix>漫画目录：</template>
      </n-input>
      <n-button :disabled="!mangaDirExist" @click="showPathInFileManager(mangaDir)">打开目录</n-button>
    </div>

    <div v-if="config" class="flex">
      <n-input v-model:value="config.outputDir" readonly placeholder="请选择漫画目录" @click="selectOutputDir">
        <template #prefix>输出目录：</template>
      </n-input>
      <n-button @click="showPathInFileManager(config.outputDir)">打开目录</n-button>
    </div>

    <manga-dir-indicator
      :manga-dir="mangaDir"
      :manga-dir-exist="mangaDirExist"
      :images-exist="imagesExist"
      :manga-dir-data-list="mangaDirDataList"
      :load-background="loadBackground"
      :auto-generate-all="autoGenerateAll"
      v-model:cropper-showing="cropperShowing"
      v-model:cropper-width="cropperWidth"
      v-model:cropper-height="cropperHeight" />

    <n-radio-group v-if="config" v-model:value="config.outputFormat">
      <n-space>
        输出格式：
        <n-radio value="Jpeg">jpg(默认)</n-radio>
        <n-tooltip placement="right-start" trigger="hover">
          <template #trigger>
            <n-radio value="Png">png</n-radio>
          </template>
          1. 以png格式输出并
          <span class="text-red">不能</span>
          提高清晰度，还会增加体积
          <br />
          2. 除非你有特殊需求，否则
          <span class="text-red">不建议</span>
          使用
          <br />
          3. 如果以png输出，建议开启体积优化
          <br />
        </n-tooltip>
      </n-space>
    </n-radio-group>
    <n-radio-group v-if="config" v-model:value="config.outputOptimize">
      <n-space>
        体积优化：
        <n-radio :value="false">关闭(默认)</n-radio>
        <n-tooltip placement="right-start" trigger="hover">
          <template #trigger>
            <n-radio :value="true">开启</n-radio>
          </template>
          1. 体积优化
          <span class="text-red">不会</span>
          影响清晰度
          <br />
          <span class="text-red">2. 显著减小png体积</span>
          <br />
          3. 对jpg影响不大，所以没有特殊需求不建议开启
          <br />
          4. 仅对黑白图片有效，彩色图片不会受影响
          <br />
        </n-tooltip>
      </n-space>
    </n-radio-group>

    <n-button :disabled="removeWatermarkButtonDisabled" type="primary" @click="removeWatermark">开始去水印</n-button>

    <n-button @click="test">测试用</n-button>

    <RemoveProgress :remove-watermark-tasks="removeWatermarkTasks" />
  </div>
  <n-modal v-model:show="cropperShowing">
    <watermark-cropper
      :manga-dir="mangaDir"
      :manga-dir-data-list="mangaDirDataList"
      :load-background="loadBackground"
      :width="cropperWidth"
      :height="cropperHeight"
      v-model:showing="cropperShowing" />
  </n-modal>
</template>
