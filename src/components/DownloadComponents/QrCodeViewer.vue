<script setup lang="ts">

import {commands, QrCodeData, QrCodeStatusData} from "../../bindings.ts";
import {ref, watch} from "vue";
import {useMessage, useNotification} from "naive-ui";

const message = useMessage();
const notification = useNotification();

const showing = defineModel<boolean>("showing", {required: true});
const biliCookie = defineModel<string>("biliCookie", {required: true});

const qrCodeData = ref<QrCodeData>();
const imgRef = ref<HTMLImageElement>();
const qrCodeStatusData = ref<QrCodeStatusData>();


watch(showing, async () => {
  if (showing.value) {
    await generateQrCode();
  } else {
    qrCodeData.value = undefined;
  }
}, {immediate: true});


async function generateQrCode() {
  const result = await commands.generateQrCode();
  if (result.status === "error") {
    notification.error({
      title: "获取二维码失败",
      description: result.error
    });
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({
      title: "获取二维码失败",
      description: response.msg
    });
    return;
  }
  qrCodeData.value = response.data;
  if (imgRef.value === undefined) {
    return;
  }
  imgRef.value.src = `data:image/jpeg;base64,${qrCodeData.value.base64}`;
  // 每隔一秒获取一次二维码状态，直到showing为false
  const interval = setInterval(async () => {
    if (!showing.value) {
      clearInterval(interval);
      return;
    }
    await getQrCodeStatusData();
    handleQrCodeStatusData();
  }, 1000);
}

async function getQrCodeStatusData() {
  if (qrCodeData.value === undefined) {
    return;
  }
  const result = await commands.getQrCodeStatusData(qrCodeData.value?.qrcodeKey);
  if (result.status === "error") {
    notification.error({
      title: "获取二维码状态失败",
      description: result.error
    });
    return;
  }
  const response = result.data;
  if (response.code !== 0) {
    notification.warning({
      title: "获取二维码状态失败",
      description: response.msg
    });
    return;
  }
  qrCodeStatusData.value = response.data;
  console.log(qrCodeStatusData.value);
}

function handleQrCodeStatusData() {
  if (qrCodeStatusData.value === undefined) {
    return;
  }

  const code = qrCodeStatusData.value.code;
  if (![0, 86101, 86090, 86038].includes(code)) {
    message.info("未知错误");
    return;
  }

  if (code === 0) {
    biliCookie.value = qrCodeStatusData.value.url.split("SESSDATA=")[1].split("&")[0];
    showing.value = false;
    message.success("登录成功");
  }

}

</script>

<template>
  <div class="flex flex-col">
    二维码状态：{{ qrCodeStatusData?.message }}
    <img ref="imgRef" src="" alt="">
  </div>
</template>

<style scoped>

</style>