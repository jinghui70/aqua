<script setup lang="ts">
// 主工具栏(有项目时常驻 AppLayout 顶部):替代 macOS 菜单远的问题,高频功能触手可及。
import { useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";

const router = useRouter();
const store = useProjectStore();
const ui = useUiStore();
</script>

<template>
  <div class="flex items-center gap-8 px-12 py-6 border-b border-gray-200 flex-shrink-0 bg-white">
    <!-- 操作组 -->
    <el-button size="small" type="primary" :disabled="store.readOnly" @click="ui.openImport">
      导入
    </el-button>
    <el-button size="small" @click="ui.openExport('ddl')">导出</el-button>

    <el-divider direction="vertical" />

    <!-- 只读切换:打开默认只读防误改,解锁可编辑 -->
    <el-button
      size="small"
      :type="store.readOnly ? 'warning' : 'default'"
      @click="store.toggleReadOnly"
    >
      {{ store.readOnly ? "🔒 解锁编辑" : "🔓 加锁" }}
    </el-button>

    <el-divider direction="vertical" />

    <!-- 管理组 -->
    <el-button size="small" @click="ui.openDataSource">数据源</el-button>
    <el-button size="small" @click="ui.openDatabaseConfig">驱动管理</el-button>
    <el-button size="small" @click="router.push('/biztype')">业务类型</el-button>
    <el-button size="small" @click="router.push('/dataset')">数据集</el-button>
  </div>
</template>
