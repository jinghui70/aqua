<script setup lang="ts">
defineProps<{ code: string }>();
</script>

<template>
  <div class="p-16">
    <div class="text-16 font-bold mb-12">表编辑: {{ code }}</div>
    <el-alert type="info" :closable="false">
      表编辑器占位 — fe-table-editor 任务实现(4 Tab: fields/index/java/json)
    </el-alert>
  </div>
</template>
