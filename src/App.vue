<script setup lang="ts">
import BackgroundMaker from "./components/BackgroundMaker.vue";
</script>

<template>
  <div>
    <background-maker :is-black="true"/>
    <background-maker :is-black="false"/>
  </div>
</template>
