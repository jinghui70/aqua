<script setup lang="ts">
// 左侧分组树容器(骨架占位,fe-group-tree 任务填实体逻辑)。
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
</script>

<template>
  <div class="h-full flex flex-col">
    <div
      class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14"
    >
      <span>表列表</span>
    </div>
    <div class="flex-1 overflow-y-auto p-8">
      <el-empty
        v-if="!store.currentProject"
        description="未打开项目"
        :image-size="60"
      />
      <div v-else class="text-12 text-gray-400 p-8">
        分组树占位 — fe-group-tree 任务实现
      </div>
    </div>
  </div>
</template>
