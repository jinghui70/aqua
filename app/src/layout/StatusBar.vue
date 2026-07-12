<script setup lang="ts">
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
</script>

<template>
  <div
    class="h-24 flex items-center px-12 border-t border-gray-200 text-12 text-gray-500 gap-16"
  >
    <span v-if="store.currentPath">{{ store.currentPath }}</span>
    <span v-else>未打开项目</span>
    <span v-if="store.currentProject">
      {{ store.currentProject.tables.length }} 表 ·
      {{ store.currentProject.groups.length }} 分组
    </span>
  </div>
</template>
