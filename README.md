# 三角洲行动同步器

**AstroBox API Level 4 / WASI 3 · Rust · 0.1.0 UI Demo · MIT**

为“三角洲行动 Vela 手表版”设计的跨屏同步工作台。首版使用明确标记的演示数据，完整体验战绩、对局详情、特勤处、离线快照、演示备份恢复和实际 JSON 文件导出。

## 界面与功能

- **战绩**：烽火地带／全面战场筛选，对局卡片显示结果、货值或积分、击杀及开始时间。
- **对局详情**：查看带出物资、数量、价值和安全箱保留物；烽火失败对局提供头、胸、腹、手臂、腿五部位的累计承伤与结束伤势。
- **特勤处**：仓库搜索与品质／分类筛选、物品详情、仓库与训练中心等级、部门任务规划演示。
- **设置**：模拟连接、手动备份、每次启动自动备份、备份预览与恢复、删除确认，以及 **1–50 份、默认 5 份**的保留策略。
- **导出**：选择任意本地备份，通过 AstroBox 系统保存窗口导出自包含 JSON，内含完整快照与 SHA-256 校验。
- **离线**：断开时保留可浏览的快照，并持续显示“未连接手表，数据未更新”和快照来源／时间。

布局根据插件容器宽度切换：手机 `<640px`、平板 `640–1023px`、电脑 `≥1024px`。视觉继承手表版的深灰绿背景、绿色主操作、低圆角和品质色卡片，信息密度适配桌面与触屏。

## 界面预览

[电脑战绩原生截图](docs/validation/screenshots/desktop-matches.png) · [手机布局预览](docs/validation/previews/phone-matches.png) · [设置预览](docs/validation/previews/desktop-settings.png)

## 体验 Demo

1. 下载 [本地导入包](dist/deltaforce-sync-0.1.0.abp)。
2. 在支持 API Level 4 的 AstroBox 中导入该文件并启用插件，打开“三角洲行动同步器”。推荐使用 AstroBox 2.2.0 及以上版本。
3. 默认即可浏览 12 条对局、20 类物资、2 项设施、6 个规划任务与 3 份演示备份。
4. 在设置里点击“模拟设备连接”，体验创建备份及自动备份；点击“查看”预览旧备份，“返回当前快照”结束预览。
5. 点击备份的“导出”，使用系统保存窗口选择位置；保存后的文件由玩家独立保管。

**数据与操作范围**：页面和备份明确标记 `mock` / `demo-extension`。首版的连接与恢复驱动本地演示状态；JSON 导出使用真实系统保存流程。原设备接入和设备端事务协议将作为后续功能实现。

## 开发

需要 Rust **1.100.0**、WASI p2 编译目标（V4 宿主同时提供 p2 标准库接口）、Python 3 和 Node.js。Windows 构建还需要 MSVC Build Tools 与 Windows SDK。

```text
rustup target add wasm32-wasip2
node scripts/cargo.mjs test --all-targets
node scripts/cargo.mjs check --target wasm32-wasip2 --lib
python scripts/build_dist.py --release --package
```

[工具链配置](rust-toolchain.toml)固定编译器版本；[依赖锁文件](Cargo.lock)固定依赖；[WIT 来源记录](wit/PROVENANCE.md)固定公开接口快照。构建脚本使用已有的系统 Cargo，或优先使用项目 `.tools` 中的隔离工具链，并把编译临时文件放在项目工具目录。

[构建与打包脚本](scripts/build_dist.py)只将清单声明的文件加入产物与 `.abp`。发布清单始终为 `api_level: 4`、`wasi_version: 3`，入口是 WebAssembly Component。

## 开发结构

| 模块 | 职责 |
| --- | --- |
| [contracts](src/contracts.rs) | 产品模型与状态契约 |
| [demo](src/demo.rs) | 确定性示例快照、对局和库存 |
| [state](src/state.rs) | 导航、筛选、连接与备份状态转换 |
| [store](src/store.rs) | 本地严格校验与上一有效版本保留 |
| [export](src/export.rs) | JSON 导出、确定性序列化及 SHA-256 校验 |
| [view](src/view.rs) / [ui](src/ui.rs) | 纯 UI 树与四页响应式呈现 |
| [runtime](src/runtime.rs) | AstroBox V4 生命周期、UI 适配和文件保存 |
| [数据测试](tests/core.rs) / [界面测试](tests/ui.rs) | 模型、交互和布局验证 |

## 数据保护与后续同步

本地演示状态位于插件自身的 `data` 目录，写入前严格校验；保存流程保留上一有效版本。确认类操作在持久化成功后提交界面状态。导出副本的保管独立于插件内的保留数量。

真实游戏数据接入的关键字段映射、历史记录和部位伤害语义见 [架构与原游戏映射](docs/architecture.md)。导出结构、散列覆盖范围及保管建议见 [备份导出格式](docs/backup-format.md)。

## 分发与协作

- 插件代码和产物提交到本仓库，代码变更通过分支 PR 交付，合并由维护者决定。
- 作者源索引 [index.txt](index.txt) 指向 `dist`。官方登记的准备步骤见 [发布说明](docs/publishing.md)。
- 构建、测试与原生验收记录见 [验证报告](docs/validation/README.md)。

## 许可与致谢

新代码采用 [MIT](LICENSE)。WIT、依赖及参考资料的署名与许可记录见 [第三方声明](THIRD_PARTY_NOTICES.md)。

原游戏参考：https://github.com/taskschd1145/deltaforce-watch-vela

AstroBox 插件文档：https://abox.run/docs/plugin-development/
