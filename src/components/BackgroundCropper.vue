<script setup lang="ts">
import {computed, onMounted, Ref, ref, watch} from "vue";
import {JpgImage, RectData} from "../types.ts";
import {open} from "@tauri-apps/plugin-dialog";
import {event} from "@tauri-apps/api";
import {TauriEvent} from "@tauri-apps/api/event";
import {commands} from "../bindings.ts";

const props = defineProps<{
  isBlack: boolean;
  mangaDir: string | undefined;
}>();
const show = defineModel<boolean>("show", {required: true});
const blackImage = defineModel<JpgImage | null>("blackImage", {required: true});
const whiteImage = defineModel<JpgImage | null>("whiteImage", {required: true});

const MASKER_OPACITY = 0.7;
const maskerValue = computed<number>(() => props.isBlack ? 255 : 0);

const canvasContainer: Ref<HTMLDivElement | null> = ref(null);
const canvas: Ref<HTMLCanvasElement | null> = ref(null);
const srcImagePath = ref<string>();
watch(srcImagePath, async () => {
  if (srcImagePath.value === undefined) {
    return;
  }
  const jpgImage = await commands.openImage(srcImagePath.value);
  if (jpgImage === null) {
    console.error(`打开图片 ${srcImagePath.value} 失败`);
    return;
  }
  srcImage.src = jpgImage.src;
  rectData = {left: 0, top: 0, right: 0, bottom: 0,};
});

const srcImage: HTMLImageElement = new Image();
let rectData: RectData = {left: 0, top: 0, right: 0, bottom: 0,};


onMounted(() => {
  if (canvas.value === null) {
    console.error("canvas is null");
    return;
  }
  console.log(blackImage);
  console.log(whiteImage);

  const ctx = canvas.value.getContext("2d")!;
  // 图片加载完成后
  srcImage.onload = () => {
    if (canvasContainer.value === null || canvas.value === null) {
      console.error("canvasContainer or canvas is null");
      return;
    }
    //设置canvas大小并显示
    canvas.value.width = srcImage.width;
    canvas.value.height = srcImage.height;
    canvas.value.style.display = "block";
    // 滚动到右下角
    canvasContainer.value.scrollTop = canvasContainer.value.scrollHeight;
    canvasContainer.value.scrollLeft = canvasContainer.value.scrollWidth;
    // 在canvas上绘制图片，并设置masker
    ctx.fillStyle = `rgba(${maskerValue.value}, ${maskerValue.value}, ${maskerValue.value}, ${MASKER_OPACITY})`;
    ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);
    ctx.globalCompositeOperation = "destination-over";
    ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height);
  };
  // 监听拖入事件
  event.listen(TauriEvent.DROP, (e: any) => {
    const hoveredElement = document.elementFromPoint(e.payload.position.x, e.payload.position.y);
    if (hoveredElement === canvasContainer.value as HTMLElement || hoveredElement === canvas.value as HTMLElement) {
      srcImagePath.value = e.payload.paths[0];
    }
  });
});

async function selectImage() {
  const fileResponse = await open({defaultPath: props.mangaDir});
  if (fileResponse !== null) {
    srcImagePath.value = fileResponse.path;
  }
}

function handleMouseDown(event: MouseEvent) {
  if (canvas.value === null) {
    console.error("canvas is null");
    return;
  }
  // 记录截图起始位置
  rectData.left = event.offsetX;
  rectData.top = event.offsetY;

  const ctx = canvas.value.getContext("2d")!;
  // 定义鼠标移动和鼠标释放事件的处理函数
  const moveEventHandler = (e: MouseEvent) => handleMouseMove(e, ctx);
  const upEventHandler = () => handleMouseUp(moveEventHandler, upEventHandler);
  // 为鼠标移动和释放事件添加事件监听器
  canvas.value.addEventListener("mousemove", moveEventHandler);
  canvas.value.addEventListener("mouseup", upEventHandler);
}

function handleMouseMove(event: MouseEvent, ctx: CanvasRenderingContext2D) {
  if (canvas.value === null) {
    console.error("canvas is null");
    return;
  }
  // 根据鼠标移动更新右下角坐标
  rectData.right = event.offsetX;
  rectData.bottom = event.offsetY;
  // 重置canvas
  ctx.reset();
  // 在整个canvas上绘制masker
  ctx.fillStyle = `rgba(${maskerValue.value}, ${maskerValue.value}, ${maskerValue.value}, ${MASKER_OPACITY})`;
  ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);
  // 切出截图区域，截图区域内的内容不受masker影响
  ctx.globalCompositeOperation = "destination-out";
  ctx.fillStyle = "#bbb";
  ctx.fillRect(
      rectData.left,
      rectData.top,
      rectData.right - rectData.left,
      rectData.bottom - rectData.top
  );
  // 重新绘制图片，使masker生效
  ctx.globalCompositeOperation = "destination-over";
  ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height);
}

function handleMouseUp(moveEventHandler: (event: MouseEvent) => void, upEventHandler: () => void) {
  // 移除鼠标移动和释放事件的监听器
  canvas.value?.removeEventListener("mousemove", moveEventHandler);
  canvas.value?.removeEventListener("mouseup", upEventHandler);
}

async function onConfirm() {
  if (srcImagePath.value === undefined) {
    console.error("请先选择图片");
    return;
  }
  await commands.generateBackground(srcImagePath.value, rectData, props.isBlack);
  show.value = false;
}

</script>

<template>
  <div>
    <n-button type="primary" @click="selectImage">选择图片</n-button>
    <div ref="canvasContainer" class="overflow-auto bg-gray" style="height: 70vh;width: 90vw">
      <span v-if="srcImagePath===undefined">使用左上角的按钮选择图片，或者直接拖拽图片到这里</span>
      <canvas class="hidden" ref="canvas" @mousedown="handleMouseDown"/>
    </div>
    <div class="flex flex-justify-end">
      <n-button type="primary" @click="onConfirm">确定</n-button>
      <n-button secondary type="primary" @click="show=false">取消</n-button>
    </div>
  </div>
</template>