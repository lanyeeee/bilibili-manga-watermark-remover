<script setup lang="ts">

const props = defineProps<{
  ok: boolean;
  content: string;
}>();

</script>

<template>
  <div>
    <span>{{ props.ok ? "✅" : "❌" }}</span>
    <span>{{ props.content }}</span>
  </div>
</template>

<style scoped>

</style>