<script setup lang="ts">
// 项目设置弹窗:中文名 + basePackage。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";

const ui = useUiStore();
const store = useProjectStore();

const name = ref("");
const basePackage = ref("");

// 打开时从当前项目载入
watch(
  () => ui.projectSettingsVisible,
  (v) => {
    if (!v) return;
    name.value = store.currentProject?.name ?? "";
    basePackage.value = store.currentProject?.basePackage ?? "";
  }
);

function save() {
  if (!store.currentProject) return;
  const pkg = basePackage.value.trim();
  if (!pkg) {
    ElMessage.warning("basePackage 不能为空");
    return;
  }
  store.currentProject.basePackage = pkg;
  const n = name.value.trim();
  store.currentProject.name = n ? n : undefined;
  ui.projectSettingsVisible = false;
  ElMessage.success("已保存");
}
</script>

<template>
  <el-dialog v-model="ui.projectSettingsVisible" title="项目设置" width="480px">
    <el-form label-width="100px">
      <el-form-item label="项目中文名">
        <el-input v-model="name" placeholder="如:订单管理系统" />
      </el-form-item>
      <el-form-item label="基础包名">
        <el-input v-model="basePackage" placeholder="com.example" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="ui.projectSettingsVisible = false">取消</el-button>
      <el-button type="primary" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>
