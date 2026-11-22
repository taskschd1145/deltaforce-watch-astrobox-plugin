# 0.1.0 Demo 验证报告

核对日期：2026-11-15。产品：**三角洲行动同步器**，Rust / AstroBox API Level 4 / WASI 3。

## 结果总览

| 层级 | 结果与证据 |
| --- | --- |
| 数据与状态测试 | **26 项通过**，覆盖快照、筛选、恢复、保留策略、自动备份去重、严格模型与导出完整性 |
| 原生布局转换 | **2 项通过**，验证 Grid→公开 Flex 转换、等宽单元及事件完整保留 |
| UI 结构与断点 | **27 项通过**，覆盖四页、九档宽度、五部位、弹层、44px 操作区域、手机底栏及显式视口 |
| 最终总计 | **55 项通过，0 失败**，见 [汇总](tests.json)与[完整日志](tests.txt) |
| WASM 组件校验 | wasm-tools 1.263.0 校验 async Component 通过，导入 V4 Host，导出 V4 lifecycle/event，见 [最终组件契约](component.wit) |
| ZIP 与发布文件 | 清单声明的 6 项文件完整，包根结构及 CRC 通过，见 [构建记录](build.json) |
| 实际 JSON 导出 | AstroBox 系统保存窗口生成文件，独立验证器确认 SHA-256、12 场对局与20类库存，见 [导出样本](exported-demo-backup.json) |
| 精确尺寸预览 | **12 张通过**，390×844、768×1024、1440×900 CSS px × 四页；逐页验证视口尺寸及水平溢出，见 [尺寸记录](viewports.json) |

## 最终产物身份

- [安装包](../../dist/deltaforce-sync-0.1.0.abp)：**269402 B**，SHA-256 `11e8aef123ad4d1160d3e2e0962ddc440727fa4afe264752cb30165ed7de147b`。
- [入口组件](../../dist/deltaforce-sync.wasm)：**951011 B**，SHA-256 `6fdb0f395953e29881b42f1073f018dc04255ad9f3e75a8f00acc437379b4151`。
- [分发清单](../../dist/manifest.json)：版本 `0.1.0`，API `4`，WASI `3`，权限 `[]`。
- 构建工具：Rust 1.100.0，wit-bindgen 0.63.0，目标 wasm32-wasip2；V4 宿主同时供 p2 标准库导入。

## 原生 AstroBox 验收

宿主：本机已安装的 **AstroBox 2.2.5 / Windows**。使用 ABP 文件关联导入，打开插件独立窗口。

实际观察与操作：

1. 插件成功加载，显示深灰绿战术界面、UI Demo、快照来源及离线状态。
2. 战绩卡点击进入对局详情，显示失败结果、保留的安全箱物资及五部位伤害示意。
3. 导航进入特勤处，显示资金、容量、品质及物品分类筛选；调整窗口宽度切换紧凑导航及手机单列。
4. 设置页显示默认保留 5 份、3 份初始备份、模拟连接及自动备份开关。
5. 点击备份“导出”，系统原生保存窗口出现；选择工程验证目录并保存，得到完整 31499 字节的 JSON 样本。
6. 使用与插件相同的严格验证器读取该文件，确认来源 `mock`、备份 `demo-backup-003`、12 场对局、20 类库存和 SHA-256 摘要。

原生检查过程中发现宿主 Grid 的布局差异，最终适配通过公开 Flex Builder 实现多列，并加入原生转换测试。手机底部导航的最后布局锚定调整由最终 UI 结构测试及精确视口预览复核。截图保留当次原生观察场景，最终产物身份以上述构建散列为准。

### 原生窗口截图

- [电脑战绩](screenshots/desktop-matches.png)
- [电脑详情](screenshots/desktop-detail.png)
- [电脑伤害与保留物资](screenshots/desktop-damage.png)
- [电脑特勤处](screenshots/desktop-services.png)
- [平板宽度特勤处](screenshots/tablet-services.png)

## 最终 UI 树尺寸预览

这些图由 Rust 的实际渲染函数及原生布局转换生成同一套 Node 树，再以精确 CSS 视口捕获，适合审阅不同尺寸的信息布局。

| 页面 | 手机 390×844 | 平板 768×1024 | 电脑 1440×900 |
| --- | --- | --- | --- |
| 战绩 | [手机](previews/phone-matches.png) | [平板](previews/tablet-matches.png) | [电脑](previews/desktop-matches.png) |
| 详情 | [手机](previews/phone-detail.png) | [平板](previews/tablet-detail.png) | [电脑](previews/desktop-detail.png) |
| 特勤处 | [手机](previews/phone-services.png) | [平板](previews/tablet-services.png) | [电脑](previews/desktop-services.png) |
| 设置 | [手机](previews/phone-settings.png) | [平板](previews/tablet-settings.png) | [电脑](previews/desktop-settings.png) |

构建预览的步骤：

```text
node scripts/cargo.mjs run --quiet --example render_previews
python scripts/render_previews.py
node scripts/capture_previews.mjs
```

## 数据范围与后续验收

本版以显式模拟数据实现 UI、演示连接、离线快照和备份交互；系统导出实际生成 JSON。原游戏目录在此次工作中用于只读参考。

后续设备同步迭代需要端到端核对游戏历史采集、开局时间、累计承伤以及设备端恢复事务。移动设备上的 AstroBox 与操作系统保存提供器将按具体设备、版本和固件独立验收。

## 可重复验证

```text
python scripts/run_tests.py
python scripts/build_dist.py --release --package
node scripts/cargo.mjs run --quiet --example verify_backup -- docs/validation/exported-demo-backup.json
```

源码与依赖身份见 [源码指纹](source-hashes.json)和[Cargo 锁文件](../../Cargo.lock)。CI 配置在 [V4 检查流程](../../.github/workflows/ci.yml)，GitHub PR 会展示远程执行状态。
