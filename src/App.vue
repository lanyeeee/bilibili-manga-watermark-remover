<script setup lang="ts">
import {onMounted, reactive, ref, Ref} from 'vue';

interface RectData {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

const srcImage = new Image();
const canvasContainer: Ref<HTMLDivElement | null> = ref(null);
const canvas: Ref<HTMLCanvasElement | null> = ref(null);
const rectData: RectData = reactive({
  left: 0,
  top: 0,
  right: 0,
  bottom: 0,
});

const MASKER_OPACITY = 0.7;
const MASKER_VALUE = 0;

onMounted(() => {
  if (canvas.value === null) {
    console.error('canvas is null')
    return;
  }

  const ctx = canvas.value.getContext('2d');
  if (ctx === null) {
    console.error('ctx is null')
    return;
  }
  // 图片加载完成后
  srcImage.onload = () => {
    if (canvasContainer.value === null || canvas.value === null) {
      console.error('canvasContainer or canvas is null')
      return;
    }
    //设置canvas大小
    canvas.value.width = srcImage.width;
    canvas.value.height = srcImage.height;
    canvasContainer.value.style.width = `${srcImage.width}px`;
    canvasContainer.value.style.height = `${srcImage.height}px`;
    canvasContainer.value.style.display = 'block';
    // 绘制图标，并设置masker
    ctx.fillStyle = `rgba(${MASKER_VALUE}, ${MASKER_VALUE}, ${MASKER_VALUE}, ${MASKER_OPACITY})`;
    ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);
    ctx.globalCompositeOperation = 'destination-over';
    ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height);
  };
});

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files === null) {
    return;
  }

  const imgFile = target.files[0];

  const fileReader = new FileReader();
  fileReader.readAsDataURL(imgFile);
  fileReader.onload = (e: ProgressEvent<FileReader>) => {
    if (e.target === null) {
      console.error('e.target is null')
      return;
    }

    srcImage.src = e.target.result as string;
  };

}

function handleMouseDown(event: MouseEvent) {
  if (canvas.value === null) {
    console.error('canvas is null')
    return;
  }
  // 记录截图起始位置
  rectData.left = event.offsetX;
  rectData.top = event.offsetY;

  const ctx = canvas.value.getContext('2d');
  if (ctx === null) {
    console.error('ctx is null')
    return;
  }
  // 定义鼠标移动和鼠标释放事件的处理函数
  const moveEventHandler = (e: MouseEvent) => handleMouseMove(e, ctx);
  const upEventHandler = () => handleMouseUp(moveEventHandler, upEventHandler);
  // 为鼠标移动和释放事件添加事件监听器
  canvas.value.addEventListener('mousemove', moveEventHandler);
  canvas.value.addEventListener('mouseup', upEventHandler);
}

function handleMouseMove(event: MouseEvent, ctx: CanvasRenderingContext2D) {
  if (canvas.value === null) {
    console.error('canvas is null')
    return;
  }
  // 根据鼠标移动更新右下角坐标
  rectData.right = event.offsetX
  rectData.bottom = event.offsetY
  // 重置canvas
  ctx.reset();
  // 在整个canvas上绘制masker
  ctx.fillStyle = `rgba(${MASKER_VALUE}, ${MASKER_VALUE}, ${MASKER_VALUE}, ${MASKER_OPACITY})`;
  ctx.fillRect(0, 0, canvas.value.width, canvas.value.height);
  // 切出截图区域，截图区域内的内容不受masker影响
  ctx.globalCompositeOperation = 'destination-out';
  ctx.fillStyle = '#bbb';
  ctx.fillRect(
      rectData.left,
      rectData.top,
      rectData.right - rectData.left,
      rectData.bottom - rectData.top
  );
  // 重新绘制图片，使masker生效
  ctx.globalCompositeOperation = 'destination-over';
  ctx.drawImage(srcImage, 0, 0, srcImage.width, srcImage.height);
}

function handleMouseUp(moveEventHandler: (event: MouseEvent) => void, upEventHandler: () => void) {
  // 移除鼠标移动和释放事件的监听器
  canvas.value?.removeEventListener('mousemove', moveEventHandler);
  canvas.value?.removeEventListener('mouseup', upEventHandler);
}


</script>

<template>
  <input type="file" id="imageFile" accept="image/*" @change="handleFileChange">
  <div ref="canvasContainer" class="canvasContainer" style="display: none;">
    <canvas ref="canvas" @mousedown="handleMouseDown"/>
  </div>
</template>
