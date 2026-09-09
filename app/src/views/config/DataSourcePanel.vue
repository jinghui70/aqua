<script setup lang="ts">
// 数据源配置面板(§6.7):列表 + 表单 + 测试连接。常驻配置中心右侧,非弹窗。
// jdbc 类库支持"主机+端口 / URL"两种配置方式(URL 模式直填完整 JDBC URL)。
import { computed, reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { useDataSourceStore, type DataSource } from "@/stores/datasource";
import { useDatabaseStore } from "@/stores/database";
import { useTauri } from "@/composables/useTauri";

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

// URL 直填模式(仅 jdbc 类库;host/port/database 与 jdbcUrl 并存于 form,切换不丢值)
const urlMode = ref(false);

// 当前 dialect 是否为 jdbc 类(native 类不提供 URL 模式,它们不认 JDBC 语法)
const isJdbcDialect = computed(
  () => dbStore.databases.find((d) => d.name === form.dialect)?.category === "jdbc"
);

// 切换类型:默认端口取后端清单(消除前端硬编码副本),切回 native 时退出 URL 模式
function onDialectChange() {
  const info = dbStore.databases.find((d) => d.name === form.dialect);
  if (info) form.port = info.defaultPort;
  if (!isJdbcDialect.value) urlMode.value = false;
}

// URL 模式提示示例(按当前 dialect;与 connector 侧 buildUrl 模板对齐,主机跟表单值联动)
const jdbcUrlPlaceholder = computed(() => {
  const host = form.host || "localhost";
  const db = form.database || "db";
  switch (form.dialect) {
    case "h2":
      return `jdbc:h2:tcp://${host}:${form.port}/${db} 或 jdbc:h2:file:/data/${db}`;
    case "oracle":
      // Oracle thin 格式与通用模板不同:@//host:port/service
      return `jdbc:oracle:thin:@//${host}:${form.port}/${db}`;
    default:
      // GenericJdbcDialect 统一模板(dm/kingbase/gbase/sqlserver 等)
      return `jdbc:${form.dialect}://${host}:${form.port}/${db}`;
  }
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
    jdbcUrl: undefined,
  });
  urlMode.value = false;
  editing.value = false;
  originalName.value = "";
}

function newSource() {
  resetForm();
  editing.value = true;
}

function editSource(ds: DataSource) {
  // jdbcUrl 显式赋值:ds 无该键时 Object.assign 不覆盖,会残留上一条数据源的 URL
  Object.assign(form, ds, { jdbcUrl: ds.jdbcUrl });
  urlMode.value = Boolean(ds.jdbcUrl);
  originalName.value = ds.sourceName;
  editing.value = true;
}

function save() {
  if (!form.sourceName.trim()) {
    ElMessage.warning("请填写数据源名称");
    return;
  }
  if (urlMode.value && !form.jdbcUrl?.trim()) {
    ElMessage.warning("请填写 JDBC URL");
    return;
  }
  const ds: DataSource = { ...form };
  // 主机模式下清掉残留的 jdbcUrl,避免重新编辑时误还原为 URL 模式
  if (!urlMode.value) ds.jdbcUrl = undefined;
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
  if (urlMode.value && !form.jdbcUrl?.trim()) {
    ElMessage.warning("请填写 JDBC URL");
    return;
  }
  testing.value = true;
  try {
    const msg = await tauri.testConnection({
      dialect: form.dialect,
      host: form.host,
      port: form.port,
      user: form.user,
      password: form.password,
      database: form.database,
      jdbcUrl: urlMode.value ? form.jdbcUrl : undefined,
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
  <div class="h-full flex flex-col">
    <div class="flex gap-16 flex-1 min-h-0">
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
            <el-select v-model="form.dialect" style="width: 100%" @change="onDialectChange">
              <el-option v-for="d in dbStore.reversible" :key="d.name" :label="d.label" :value="d.name" />
            </el-select>
          </el-form-item>
          <el-form-item v-if="isJdbcDialect" label="配置方式">
            <el-radio-group v-model="urlMode">
              <el-radio-button :value="false">主机 + 端口</el-radio-button>
              <el-radio-button :value="true">URL</el-radio-button>
            </el-radio-group>
          </el-form-item>
          <template v-if="urlMode">
            <el-form-item label="JDBC URL">
              <el-input v-model="form.jdbcUrl" :placeholder="jdbcUrlPlaceholder" />
            </el-form-item>
          </template>
          <template v-else>
            <el-form-item label="主机">
              <el-input v-model="form.host" />
            </el-form-item>
            <el-form-item label="端口">
              <el-input-number v-model="form.port" :min="1" :max="65535" :controls="false" style="width: 120px" />
            </el-form-item>
            <el-form-item label="数据库">
              <el-input v-model="form.database" />
            </el-form-item>
          </template>
          <el-form-item label="用户名">
            <el-input v-model="form.user" />
          </el-form-item>
          <el-form-item label="密码">
            <el-input v-model="form.password" type="password" show-password />
          </el-form-item>
          <el-form-item>
            <el-button :loading="testing" @click="testConnection">测试连接</el-button>
            <el-button type="primary" @click="save">保存</el-button>
          </el-form-item>
        </el-form>
        <el-empty v-else description="选择或新建数据源" />
      </div>
    </div>
  </div>
</template>
