<script setup lang="ts">
// 主工具栏(有项目时常驻 AppLayout 顶部):替代 macOS 菜单远的问题,高频功能触手可及。
// 布局:文件组[保存] │ 生成组[导入][导出] │ 管理组[配置][数据集][驱动] ──spacer── [只读锁(最右)]
import { useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { useMenuActions } from "@/composables/useMenuActions";

const router = useRouter();
const store = useProjectStore();
const ui = useUiStore();
// 保存复用菜单动作的 doSave(无路径走保存对话框),按钮 + Cmd+S 单一实现双入口。
const { doSave } = useMenuActions();
</script>

<template>
  <div class="flex items-center gap-6 px-12 py-8 border-b border-gray-200 flex-shrink-0 bg-white">
    <!-- 文件组:保存(仅可编辑时显示;无改动置灰;dirty 红点提示) -->
    <el-button
      v-if="!store.readOnly"
      size="default"
      type="primary"
      :disabled="!store.dirty"
      class="relative"
      @click="doSave()"
    >
      <span class="i-mdi-content-save w-16 h-16 mr-4" />
      保存
      <span
        v-if="store.dirty"
        class="absolute -top-2 -right-2 w-8 h-8 rounded-full bg-red-500"
      />
    </el-button>

    <el-divider v-if="!store.readOnly" direction="vertical" />

    <!-- 生成组 -->
    <el-button size="default" :disabled="store.readOnly" @click="ui.openImport">
      <span class="i-mdi-database-import w-16 h-16 mr-4" />
      导入
    </el-button>
    <el-dropdown trigger="click" @command="(k: string) => ui.openExport(k as any)">
      <el-button size="default">
        <span class="i-mdi-database-export w-16 h-16 mr-4" />
        导出
        <span class="i-mdi-menu-down w-16 h-16" />
      </el-button>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item command="ddl">DDL</el-dropdown-item>
          <el-dropdown-item command="diff">diff (ALTER)</el-dropdown-item>
          <el-dropdown-item command="strconst">字符串变量</el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>

    <el-divider direction="vertical" />

    <!-- 管理组 -->
    <el-button size="default" @click="router.push('/config')">
      <span class="i-mdi-cog w-16 h-16 mr-4" />
      配置
    </el-button>
    <el-button size="default" @click="router.push('/dataset')">
      <span class="i-mdi-table w-16 h-16 mr-4" />
      数据集
    </el-button>
    <el-button size="default" @click="ui.openDatabaseConfig">
      <span class="i-mdi-database-cog w-16 h-16 mr-4" />
      驱动管理
    </el-button>

    <div class="flex-1" />

    <!-- 只读切换(最右):打开默认只读防误改,解锁可编辑 -->
    <el-button
      size="default"
      :type="store.readOnly ? 'warning' : 'default'"
      @click="store.toggleReadOnly"
    >
      <span :class="store.readOnly ? 'i-mdi-lock' : 'i-mdi-lock-open-variant'" class="w-16 h-16 mr-4" />
      {{ store.readOnly ? "解锁编辑" : "加锁" }}
    </el-button>
  </div>
</template>
