<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

const msg = ref("连接中...");
invoke<string>("greet", { name: "aqua" })
  .then((m) => (msg.value = m))
  .catch((e) => (msg.value = "错误: " + e));
</script>

<template>
  <div style="padding: 24px; font-family: sans-serif">
    <h1>aqua v2</h1>
    <p>{{ msg }}</p>
    <el-button type="primary">element-plus 链路验证</el-button>
  </div>
</template>
