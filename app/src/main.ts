import { createApp } from "vue";
import { createPinia } from "pinia";
import "@unocss/reset/tailwind.css";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import "virtual:uno.css";
import "./styles/global.css";
import App from "./App.vue";
import { router } from "./router";

createApp(App)
  .use(createPinia())
  .use(router)
  .use(ElementPlus)
  .mount("#app");

// 生产环境禁用 webview 右键菜单(Windows 右键出现浏览器菜单)
if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (e) => e.preventDefault());
}
