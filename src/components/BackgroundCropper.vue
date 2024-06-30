<script setup lang="ts">
import {computed, onMounted, Ref, ref} from "vue";
import {RectData} from "../types.ts";
import {invoke} from "@tauri-apps/api/core";
import {open} from "@tauri-apps/plugin-dialog";

const props = defineProps<{
  isBlack: boolean;
  mangaDir: string | undefined;
}>();
const show = defineModel<boolean>("show", {required: true});

const MASKER_OPACITY = 0.7;
const maskerValue = computed(() => props.isBlack ? 255 : 0);

const canvasContainer: Ref<HTMLDivElement | null> = ref(null);
const canvas: Ref<HTMLCanvasElement | null> = ref(null);
const srcImage: HTMLImageElement = new Image();
let rectData: RectData = {left: 0, top: 0, right: 0, bottom: 0,};
let srcImagePath: string = "";


onMounted(() => {
  if (canvas.value === null) {
    console.error("canvas is null");
    return;
  }

  const ctx = canvas.value.getContext("2d");
  if (ctx === null) {
    console.error("ctx is null");
    return;
  }
  // 图片加载完成后
  srcImage.onload = () => {
    if (canvasContainer.value === null || canvas.value === null) {
      console.error("canvasContainer or canvas is null");
      return;
    }
    //设置canvas大小
    canvas.value.width = srcImage.width;
    canvas.value.height = srcImage.height;
    canvasContainer.value.style.display = "block";
    canvasContainer.value.scrollTop = canvasContainer.value.scrollHeight;
    canvasContainer.value.scrollLeft = canvasContainer.value.scrollWidth;
    // 绘制图片，并设置masker
    ctx.fillStyle = `rgba(${maskerValue.value}, ${maskerValue.value}, ${maskerValue.value}, ${MASKER_OPACITY})`;
    ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);
    ctx.globalCompositeOperation = "destination-over";
    ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height);
  };
});

async function selectImage() {
  const fileResponse = await open({defaultPath: props.mangaDir});
  if (fileResponse === null) {
    // 用户取消选择文件
    return;
  }

  srcImagePath = fileResponse.path;

  const imageData: ArrayBuffer = await invoke("read_file", {path: srcImagePath});
  // 转换为 Base64
  const base64 = btoa(
      new Uint8Array(imageData).reduce(
          (data, byte) => data + String.fromCharCode(byte),
          ""
      )
  );

  // 创建数据 URL 并更新 imageSrc
  srcImage.src = `data:image/jpeg;base64,${base64}`;
}

function handleMouseDown(event: MouseEvent) {
  if (canvas.value === null) {
    console.error("canvas is null");
    return;
  }
  // 记录截图起始位置
  rectData.left = event.offsetX;
  rectData.top = event.offsetY;

  const ctx = canvas.value.getContext("2d");
  if (ctx === null) {
    console.error("ctx is null");
    return;
  }
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
  await invoke("generate_background", {imagePath: srcImagePath, rectData: rectData, isBlack: props.isBlack});
  show.value = false;
}

</script>

<template>
  <div>
    <button @click="selectImage">选择图片</button>
    <div ref="canvasContainer" class="overflow-auto hidden" style="height: 70vh;width: 90vw">
      <canvas ref="canvas" @mousedown="handleMouseDown"/>
    </div>
    <n-button type="primary" @click="onConfirm">确定</n-button>
    <n-button type="info" @click="show=false">取消</n-button>
  </div>
</template>