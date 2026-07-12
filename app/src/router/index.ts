import { createRouter, createWebHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/welcome" },
  {
    path: "/welcome",
    name: "welcome",
    component: () => import("@/views/Welcome.vue"),
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
