<script setup lang="ts">
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
</script>

<template>
  <div class="h-full flex flex-col items-center justify-center text-gray-400">
    <div class="text-24 mb-12">aqua v2</div>
    <p v-if="!store.currentProject" class="text-14">
      从「文件」菜单新建或打开项目
    </p>
    <p v-else class="text-14">从左侧分组树选择表,或新建表</p>
  </div>
</template>
