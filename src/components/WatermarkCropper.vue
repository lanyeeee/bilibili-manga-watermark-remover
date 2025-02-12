<script setup lang="ts">
import { commands, JpgImageInfo, MangaDirData, RectData } from '../bindings.ts'
import { computed, onMounted, ref, watch } from 'vue'
import { useMessage, useNotification } from 'naive-ui'

const props = defineProps<{
  mangaDir: string | undefined
  mangaDirDataList: MangaDirData[]
  loadBackground: () => Promise<void>
  width: number
  height: number
}>()

const notification = useNotification()
const message = useMessage()

const showing = defineModel<boolean>('showing', { required: true })

const MASKER_OPACITY = 0.7
const srcImage: HTMLImageElement = new Image()

let jpgImageInfos: JpgImageInfo[] = []

const rectData = ref<RectData | null>(null)
const canvasContainer = ref<HTMLDivElement>()
const canvas = ref<HTMLCanvasElement>()
const srcImagePath = ref<string>()
const isDarkMasker = ref<boolean>(true)
const generating = ref<boolean>(false)

// masker的值，深色遮罩为0，浅色遮罩为255
const maskerValue = computed<number>(() => (isDarkMasker.value ? 0 : 255))
const matchingJpgImageInfos = computed<JpgImageInfo[]>(() =>
  jpgImageInfos.filter((info) => info.width == props.width && info.height == props.height),
)

// 监听 srcImagePath 的变化，当路径变化时，加载对应的图片
watch(srcImagePath, async () => {
  if (srcImagePath.value === undefined) {
    return
  }
  // 打开图片
  const result = await commands.openImage(srcImagePath.value)
  if (result.status === 'error') {
    notification.error({ title: '打开图片失败', description: result.error })
    return
  }
  srcImage.src = `data:image/jpeg;base64,${result.data.base64}`
  rectData.value = null
})
// 监听 mangaDir 的变化，当路径变化时，获取对应路径下的所有jpg图片信息，并从中随机选择一张图片，将其路径赋值给srcImagePath
watch(
  () => props.mangaDir,
  async () => {
    if (props.mangaDir === undefined) {
      return
    }
    // 获取mangaDir下所有jpg图片信息
    jpgImageInfos = await commands.getJpgImageInfos(props.mangaDir)
    // 随机选择一张图片，将其路径赋值给srcImagePath
    srcImagePath.value = getRandomJpgImageInfo()?.path
  },
  { immediate: true },
)
// 监听 isDarkMasker 的变化，当遮罩颜色变化时，重新绘制canvas
watch(isDarkMasker, () => {
  if (canvas.value === undefined) {
    message.error('canvas is undefined')
    return
  }
  // 重新绘制canvas
  drawImageAndMasker()
})

onMounted(() => {
  if (canvas.value === undefined) {
    message.error('canvas is undefined')
    return
  }
  // 每张图片加载完成后
  srcImage.onload = async () => {
    if (canvasContainer.value === undefined || canvas.value === undefined) {
      message.error('canvasContainer or canvas is undefined')
      return
    }
    console.log(`图片${srcImage.width}x${srcImage.height}加载完成`)
    // 设置canvas大小
    canvas.value.width = srcImage.width
    canvas.value.height = srcImage.height
    // 滚动到右下角
    canvasContainer.value.scrollTop = canvas.value.height
    canvasContainer.value.scrollLeft = canvas.value.width
    // 在canvas上绘制图片和masker
    drawImageAndMasker()
  }
})

// 随机从筛选后的jpg图片信息中选择一张图片
function getRandomJpgImageInfo(): JpgImageInfo | null {
  if (matchingJpgImageInfos.value.length === 0) {
    return null
  }
  return matchingJpgImageInfos.value[Math.floor(Math.random() * matchingJpgImageInfos.value.length)]
}

function handleMouseDown(event: MouseEvent) {
  if (canvas.value === undefined) {
    return
  }
  // 记录截图起始位置
  rectData.value = { left: event.offsetX, top: event.offsetY, right: event.offsetX, bottom: event.offsetY }
  // 为鼠标移动和释放事件添加事件监听器
  canvas.value.addEventListener('mousemove', handleMouseMove)
  canvas.value.addEventListener('mouseup', handleMouseUp)
}

function handleMouseMove(event: MouseEvent) {
  if (canvas.value === undefined || rectData.value === null) {
    return
  }
  // 根据鼠标移动更新右下角坐标
  rectData.value.right = event.offsetX
  rectData.value.bottom = event.offsetY
  // 重新绘制canvas
  drawImageAndMasker()
}

function handleMouseUp() {
  // 移除鼠标移动和释放事件的监听器
  canvas.value?.removeEventListener('mousemove', handleMouseMove)
  canvas.value?.removeEventListener('mouseup', handleMouseUp)
}

// 在canvas上绘制图片和masker
function drawImageAndMasker() {
  if (canvas.value === undefined) {
    return
  }
  // 获取canvas上下文
  const ctx = canvas.value.getContext('2d')!
  // 重置canvas
  ctx.reset()
  // 在整个canvas上绘制masker
  ctx.fillStyle = `rgba(${maskerValue.value}, ${maskerValue.value}, ${maskerValue.value}, ${MASKER_OPACITY})`
  ctx.fillRect(0, 0, canvas.value.width, canvas.value.height)
  // 如果rectData不为null，切出rectData所代表的区域
  if (rectData.value !== null) {
    ctx.globalCompositeOperation = 'destination-out'
    ctx.fillStyle = '#bbb'
    ctx.fillRect(
      rectData.value.left,
      rectData.value.top,
      rectData.value.right - rectData.value.left,
      rectData.value.bottom - rectData.value.top,
    )
  }
  // 绘制图片，使masker生效
  ctx.globalCompositeOperation = 'destination-over'
  ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height)
}

async function generateBackground() {
  if (srcImagePath.value === undefined) {
    message.error('图片未加载')
    return
  }
  if (rectData.value === null) {
    message.error('请截取图片中的水印')
    return
  }
  if (props.mangaDir === undefined) {
    message.error('请选择漫画文件夹')
    return
  }

  generating.value = true
  const width = props.width
  const height = props.height
  const result = await commands.generateBackground(props.mangaDir, rectData.value, width, height)
  await props.loadBackground()
  if (result.status === 'error') {
    notification.error({ title: '生成背景水印图失败', description: result.error })
    generating.value = false
    return
  }

  message.success('生成背景水印图成功')
  showing.value = false
}

async function changeImage() {
  srcImagePath.value = getRandomJpgImageInfo()?.path
}
</script>

<template>
  <div>
    <n-button type="primary" @click="changeImage">换一张</n-button>
    <n-switch v-model:value="isDarkMasker">
      <template #checked>深色遮罩</template>
      <template #unchecked>浅色遮罩</template>
    </n-switch>
    <div ref="canvasContainer" class="overflow-auto bg-gray" style="height: 40vh; width: 95vw">
      <canvas ref="canvas" @mousedown="handleMouseDown" />
    </div>
    <div class="flex flex-justify-end">
      <n-button :loading="generating" :disabled="rectData === null" type="primary" @click="generateBackground">
        生成背景水印图
      </n-button>
    </div>
  </div>
</template>
