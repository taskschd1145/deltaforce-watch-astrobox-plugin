# 作者仓库发布与官方登记准备

依据：https://abox.run/docs/creator-tools/resource-management/plugin-upload

## 本项目的作者源

本仓库根目录 [index.txt](../index.txt) 内容为：

```text
dist
```

目录路径保持相对形式，包体目录中放齐 [清单](../manifest.json)声明的入口 WASM、PNG 图标及全部附加文件。[构建脚本](../scripts/build_dist.py)生成 `.abp` 供本地导入，并核对 ZIP 成员和校验值。

清单约定：名称“三角洲行动同步器”、版本 0.1.0、`api_level: 4`、`wasi_version: 3`，主页为本仓库。每次发布同步递增清单与源码版本，重新构建、验证并通过 PR 交付。

## GitHub 协作

唯一代码提交目标：

https://github.com/taskschd1145/deltaforce-watch-astrobox-plugin

首次获准使用 main 空提交建立 PR 基线。功能与产物在分支提交，通过 gh CLI 创建对应 PR，合并由维护者决定。AstroBox 的公开仓库作为接口及流程资料来源。

## 以后正式登记到官方源

当维护者明确决定登记时，官方源需要的作者 raw 基地址为：

```text
https://raw.githubusercontent.com/taskschd1145/deltaforce-watch-astrobox-plugin/refs/heads/main/
```

官方聚合流程先读取这个基地址下的作者索引，再读取其中列出的产物目录和清单。作者索引的目录名与官方索引的 raw 基地址承担不同职责。

官方登记 PR 说明应包括插件名称与简介、API Level 4 / WASI 3、权限清单、功能范围、已知问题与测试情况。合入后官方 Action 在 push 或约 4 小时的计划任务中聚合数据。

## 发布检查表

- 作者分支中的 raw 清单与每个声明文件可访问。
- 产物全部来自当前源码和依赖锁定版本。
- 发布目录包含入口、图标、许可与声明的附加资源。
- `.abp` 根目录直接包含清单与入口，ZIP 校验通过。
- UI 演示与真实功能的来源说明在清单、README 和界面中一致。
- 官方登记步骤由维护者单独指定后执行。
