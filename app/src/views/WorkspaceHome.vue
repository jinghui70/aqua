<script setup lang="ts">
// 工作区首页:有项目但未打开任何表时的引导空状态。
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();
</script>

<template>
  <div class="h-full flex flex-col items-center justify-center text-gray-400">
    <div class="text-16 mb-8">
      {{ store.currentProject?.basePackage }}
    </div>
    <p class="text-14">从左侧分组树选择或新建表开始编辑</p>
    <p class="text-13 mt-4">
      配置菜单可管理业务类型 / 枚举 / 数据源 / 数据集
    </p>
  </div>
</template>
