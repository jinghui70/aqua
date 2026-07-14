<script setup lang="ts">
// 数据源配置弹窗(§6.7):列表 + 表单 + 测试连接。
import { reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { useUiStore } from "@/stores/ui";
import { useDataSourceStore, type DataSource } from "@/stores/datasource";
import { useDatabaseStore } from "@/stores/database";
import { useTauri } from "@/composables/useTauri";

const ui = useUiStore();
const dsStore = useDataSourceStore();
const tauri = useTauri();
const dbStore = useDatabaseStore();

// 当前编辑的表单(新增或编辑)
const editing = ref(false);
const originalName = ref("");
const form = reactive<DataSource>({
  sourceName: "",
  dialect: "mysql",
  host: "localhost",
  port: 3306,
  user: "",
  password: "",
  database: "",
});

function resetForm() {
  Object.assign(form, {
    sourceName: "",
    dialect: "mysql",
    host: "localhost",
    port: 3306,
    user: "",
    password: "",
    database: "",
  });
  editing.value = false;
  originalName.value = "";
}

function newSource() {
  resetForm();
  editing.value = true;
}

function editSource(ds: DataSource) {
  Object.assign(form, ds);
  originalName.value = ds.sourceName;
  editing.value = true;
}

function save() {
  if (!form.sourceName.trim()) {
    ElMessage.warning("请填写数据源名称");
    return;
  }
  const ds: DataSource = { ...form };
  if (originalName.value) {
    dsStore.update(originalName.value, ds);
    ElMessage.success("已更新");
  } else {
    const err = dsStore.add(ds);
    if (err) {
      ElMessage.error(err);
      return;
    }
    ElMessage.success("已添加");
  }
  resetForm();
}

function removeSource(name: string) {
  dsStore.remove(name);
  if (originalName.value === name) resetForm();
}

const testing = ref(false);
async function testConnection() {
  testing.value = true;
  try {
    const msg = await tauri.testConnection({
      dialect: form.dialect,
      host: form.host,
      port: form.port,
      user: form.user,
      password: form.password,
      database: form.database,
    });
    ElMessage.success(msg);
  } catch {
    /* 已提示 */
  } finally {
    testing.value = false;
  }
}
</script>

<template>
  <el-dialog v-model="ui.dataSourceVisible" title="数据源配置" width="720px">
    <div class="flex gap-16" style="height: 400px">
      <!-- 左列表 -->
      <div class="w-200 border-r border-gray-200 flex flex-col flex-shrink-0">
        <el-button size="small" type="primary" class="mb-8" @click="newSource">
          + 新建数据源
        </el-button>
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="ds in dsStore.sources"
            :key="ds.sourceName"
            class="flex items-center justify-between px-8 py-6 cursor-pointer text-13 hover:bg-gray-100"
            :class="{ 'bg-blue-50': ds.sourceName === originalName }"
            @click="editSource(ds)"
          >
            <span>{{ ds.sourceName }} <span class="text-gray-400">({{ ds.dialect }})</span></span>
            <el-button size="small" link type="danger" @click.stop="removeSource(ds.sourceName)">删</el-button>
          </div>
          <el-empty v-if="!dsStore.sources.length" description="暂无" :image-size="50" />
        </div>
      </div>

      <!-- 右表单 -->
      <div class="flex-1 overflow-y-auto">
        <el-form v-if="editing" label-width="90px">
          <el-form-item label="名称">
            <el-input v-model="form.sourceName" placeholder="dev / test / prod" />
          </el-form-item>
          <el-form-item label="类型">
            <el-select v-model="form.dialect" style="width: 100%">
              <el-option v-for="d in dbStore.reversible" :key="d.name" :label="d.label" :value="d.name" />
            </el-select>
          </el-form-item>
          <el-form-item label="主机">
            <el-input v-model="form.host" />
          </el-form-item>
          <el-form-item label="端口">
            <el-input-number v-model="form.port" :min="1" :max="65535" :controls="false" style="width: 120px" />
          </el-form-item>
          <el-form-item label="用户名">
            <el-input v-model="form.user" />
          </el-form-item>
          <el-form-item label="密码">
            <el-input v-model="form.password" type="password" show-password />
          </el-form-item>
          <el-form-item label="数据库">
            <el-input v-model="form.database" />
          </el-form-item>
          <el-form-item>
            <el-button :loading="testing" @click="testConnection">测试连接</el-button>
            <el-button type="primary" @click="save">保存</el-button>
          </el-form-item>
        </el-form>
        <el-empty v-else description="选择或新建数据源" />
      </div>
    </div>
  </el-dialog>
</template>
