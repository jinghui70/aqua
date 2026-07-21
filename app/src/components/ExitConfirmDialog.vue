<script setup lang="ts">
// 退出/关闭确认对话框(三按钮:保存/不保存/取消)。
// 由 ui.openExitConfirm() 触发,resolveExitConfirm(action) 解决 Promise。
import { useUiStore } from "@/stores/ui";

const ui = useUiStore();
</script>

<template>
  <el-dialog
    v-model="ui.exitConfirmVisible"
    title="提示"
    width="420px"
    :close-on-click-modal="false"
    :show-close="false"
    :close-on-press-escape="false"
  >
    <span class="text-14">有未保存改动,是否保存?</span>
    <template #footer>
      <el-button @click="ui.resolveExitConfirm('cancel')">取消</el-button>
      <el-button @click="ui.resolveExitConfirm('discard')">不保存</el-button>
      <el-button type="primary" @click="ui.resolveExitConfirm('save')">保存</el-button>
    </template>
  </el-dialog>
</template>
