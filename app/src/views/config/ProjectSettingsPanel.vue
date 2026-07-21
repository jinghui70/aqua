<script setup lang="ts">
// 项目设置面板(配置中心):中文名 + basePackage。从 ProjectSettingsDialog 迁移。
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();

const name = ref("");
const basePackage = ref("");

// 从当前项目载入(打开配置中心时同步)
watch(
  () => store.currentProject,
  (p) => {
    name.value = p?.name ?? "";
    basePackage.value = p?.basePackage ?? "";
  },
  { immediate: true }
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
  ElMessage.success("已保存");
}
</script>

<template>
  <el-form label-width="100px" :disabled="store.readOnly" class="max-w-480">
    <el-form-item label="项目中文名">
      <el-input v-model="name" placeholder="如:订单管理系统" />
    </el-form-item>
    <el-form-item label="基础包名">
      <el-input v-model="basePackage" placeholder="com.example" />
    </el-form-item>
    <el-form-item>
      <el-button type="primary" :disabled="store.readOnly" @click="save">保存</el-button>
    </el-form-item>
  </el-form>
</template>
