<script setup lang="ts">
// 新建项目弹窗:项目中文名 + basePackage。
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";

const ui = useUiStore();
const store = useProjectStore();
const router = useRouter();

const name = ref("");
const basePackage = ref("com.example");

// 每次打开重置为默认值
watch(
  () => ui.newProjectVisible,
  (v) => {
    if (!v) return;
    name.value = "";
    basePackage.value = "com.example";
  }
);

function confirm() {
  const pkg = basePackage.value.trim();
  if (!pkg) {
    ElMessage.warning("basePackage 不能为空");
    return;
  }
  store.newProject(name.value, pkg);
  ui.newProjectVisible = false;
  router.push("/");
  ElMessage.success("已新建项目");
}
</script>

<template>
  <el-dialog v-model="ui.newProjectVisible" title="新建项目" width="480px" :close-on-click-modal="false">
    <el-form label-width="100px">
      <el-form-item label="项目中文名">
        <el-input v-model="name" placeholder="如:订单管理系统" />
      </el-form-item>
      <el-form-item label="基础包名">
        <el-input v-model="basePackage" placeholder="com.example" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="ui.newProjectVisible = false">取消</el-button>
      <el-button type="primary" @click="confirm">创建</el-button>
    </template>
  </el-dialog>
</template>
