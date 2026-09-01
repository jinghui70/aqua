// 产品版本号,由 vite.config.ts 的 define 在构建期注入(读 package.json,release 时 version-sync 同步)。
declare const __APP_VERSION__: string;

export const APP_VERSION: string = __APP_VERSION__;
