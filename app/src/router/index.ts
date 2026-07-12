import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    // 工作区首页(有项目但未开任何标签时的空状态)
    path: "/",
    name: "home",
    component: () => import("@/views/WorkspaceHome.vue"),
  },
  {
    path: "/table/:code",
    name: "table",
    component: () => import("@/views/TableEditor.vue"),
    props: true,
  },
  {
    path: "/biztype",
    name: "biztype",
    component: () => import("@/views/BizTypeManage.vue"),
  },
  {
    path: "/enum",
    name: "enum",
    component: () => import("@/views/EnumManage.vue"),
  },
  {
    path: "/dataset",
    name: "dataset",
    component: () => import("@/views/DatasetManage.vue"),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
