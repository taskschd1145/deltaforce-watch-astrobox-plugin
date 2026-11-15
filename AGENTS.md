# AstroBox 插件开发约定

## 核心规则（优先遵守）

### 正面解释工作成果

避免否定式表达：当你解释你的工作结果时，优先使用正面描述，比如“这份工作是什么，代表了什么，可以用于证明什么”，而不要使用否定描述，比如“这份工作不是什么，不代表什么，不应用于什么，不应被视为什么”。永远不要用否定描述来解释你的工作内容，否定描述只应用于判断。

### 形成方向后立即行动

在思考时，不要在一点上反复纠结。如果你已经想出两三条可能的方向，就先动手——修改、测试、上网搜索都算。如果这些都没达到预期的目标，再重新思考。

## 1. 适用范围与项目边界

- 本文件用于指导本仓库后续的 AstroBox 插件开发，文档核对日期：**2026-11-15**。
- **用户确定的固定技术目标：本项目统一编写 AstroBox API Level 4 插件，使用 `api_level: 4`、`wasi_version: 3` 和 V4 WIT 接口。** V2 Rust 教程作为工程与概念基础，实际代码按 V4 实现。
- 当前产品为“三角洲行动同步器”0.1.0 UI Demo，包含战绩、对局详情、特勤处、设置四页；使用模拟连接与本地演示快照，支持实际系统保存窗口导出 JSON。
- 本仓库是 **AstroBox 宿主侧插件**项目，不是设备端 Vela 快应用工程，也不是 DeepSeek Harness 插件。插件分发包 `.abp`、插件执行体 `.wasm` 与设备端安装包 `.rpk` 不可混为一谈。
- 关联的 deltaforce-watch-vela 历史会话仅用于理解业务背景；不得把其中的版本号、签名材料、RPK 包体门槛、发布授权或设备验证结果直接沿用为本插件的约定。
- 用户已批准界面 Demo、演示备份与真实文件导出、原生宿主验收，以及向自己的仓库通过 PR 交付。设备接入与设备端改动作为后续独立迭代。原游戏目录始终用于只读参考。
- 新代码采用 MIT。唯一代码提交目标为 `taskschd1145/deltaforce-watch-astrobox-plugin`，使用 gh CLI 创建对应 PR；首次 main 空基线已获准，其后代码提交均配套 PR，合并由维护者决定。AstroBox 相关仓库用于只读查阅，上游提交等待用户明确指示。

## 2. 事实来源与版本选择

1. **本项目的 API Level 固定为 4**。开发前确认目标 AstroBox 客户端版本、平台和所需 V4 功能。[官方对应表][api-level]将 Level 4 的最低 AstroBox 版本列为 **2.2.0**；具体新增接口按目标宿主实际支持情况核对，客户端支持 V4 作为验收前提。
2. 官方文档入口目前默认跳转到 **API Level 4**。Level 4 是运行时层面的升级；本项目沿用插件工程、WIT 契约、事件订阅及构建流程等基础，采用 V4 命名空间、导出和异步语义。目标客户端版本不足时提供升级提示，保持本项目 V4 定位。[概览][overview] [Level 4 变更][v4-new]
3. Level 4 使用新版 wasmtime 和 **WASI Preview 3**，清单应匹配 `api_level: 4`、`wasi_version: 3`；Level 2/3 保留旧运行时，两套接口不能混用。
4. Level 4 的基础世界是 `psys-world-v4`，宿主包为 `astrobox:psys-host-v4`，插件导出包为 `astrobox:psys-plugin-v4`。扩展功能可能要求不同的 world，必须核对构建配置与最终导出。[官方 WIT 世界定义][wit-world]
5. V2 Rust 教程是重点学习资料：用它掌握 Rust 组件、项目结构、日志、UI 和事件流程；函数签名、命名空间、异步桥接及资源限制按 V4 文档和绑定落实。V1 文档已过时，不作为新项目模板。
6. 依据目标宿主、锁定的 WIT 与实际生成绑定核实签名；官方文档、模板、WIT 与宿主行为不一致时记录差异，先核实，不凭记忆编造 API。本文不是完整 API 参考，链接指向的 main 分支也会变化。

## 3. 语言、工具链与异步语义

官方支持 Rust、Python、JavaScript；不要因为插件可以用 JS，就把它当作 Node.js 脚本或浏览器网页运行。[语言选择][language]

### Rust（重点必读，按 V4 实现）

**用户指定的重点资料：[Rust 插件开发上手教程](https://abox.run/docs/plugin-development/v2/getstarted/rust)。Rust 开发先阅读此文掌握组件与工程基础，再用 V4 WIT 和生成绑定落实代码。** 本项目的实现目标始终为 **API Level 4 / WASI 3**。

#### 从教程掌握的工程基础

- Rust 插件作为 `lib` 编译成 WebAssembly Component，Host 与 Plugin 通过 WIT 定义的强类型组件接口协作。
- 准备 Rust、Python 3；Windows 按 Rust 安装指引准备 MSVC。从[官方 Rust 模板](https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Template-Rust)学习，并完整初始化 `wit/` 子模块。
- 先完成一遍基础工程阅读，再把清单、world、绑定和 Guest 实现统一配置为 V4，完成首次构建与宿主加载闭环，然后开始业务改动。
- 教程中的关键文件及职责如下，后续工程以实际采用的模板结构为准：

```text
Cargo.toml               # Rust 工程、依赖和构建配置
src/lib.rs               # 插件入口、绑定生成与 Guest 实现
src/logger.rs            # tracing 日志初始化
scripts/build_dist.py    # 构建与打包辅助脚本
wit/                     # Host / Plugin 的 WIT 子模块
```

#### 本项目的 V4 配置与构建闭环

1. 清单配置为 `api_level: 4`、`wasi_version: 3`。
2. 选择支持 Component Model async 的 `wit-bindgen` 和兼容的 Rust 工具链，记录依赖版本与实际使用的 WIT commit。
3. 使用基础世界 `psys-world-v4`；HTTP handler、资源接管等扩展能力选对应的 V4 world。
4. 宿主调用使用 `astrobox:psys-host-v4`，插件导出使用 `astrobox:psys-plugin-v4`，同步调整所有 Guest 实现。
5. 运行工程中已核实的构建与打包脚本，检查最终 Component 的 V4 导入导出，再在支持 V4 的 AstroBox 中完成加载验证。

基础绑定生成形式参考 V4 文档：

```rust
wit_bindgen::generate!({
    path: "wit",
    world: "psys-world-v4",
    generate_all,
});
```

教程给出的构建脚本调用形式可用于适配后的 V4 工程，执行时核对实际脚本参数和输出：

```text
# Debug 构建，输出到 dist/
python scripts/build_dist.py

# Release 构建并打包 .abp
python scripts/build_dist.py --release --package
```

教程的 Rust 编译 target 示例为 `wasm32-wasip2`；V4 宿主使用 WASI p3，并同时提供 p2 标准库接口。编译 target 与语言运行时的选择按 V4 工程实际工具链验证，清单保持 **WASI 3 / API 4**。

#### 本项目的 V4 生命周期、异步与事件规则

- `on-load` 在 V4 中是 `async func`，Rust 按生成的 async Guest 接口实现；初始化日志后，直接 await 必要的异步 Host 调用和事件注册。
- `on-event`、`on-ui-event` 等事件入口按 V4 Guest 签名实现和返回结果；业务按事件类型或 UI 事件 ID 分发。
- 事件采用 **Register → Event** 订阅流程：先注册，再处理事件；收到的载荷先做类型、长度与协议校验。
- `wit_bindgen::generate!` 负责生成 Host 绑定、Guest trait 及相应桥接代码；业务代码围绕生成接口实现。
- V4 异步 Host 调用使用普通 `.await`；UI 建树、`ui.render`、定时器建立/取消等同步 `func` 按原签名调用。
- 加载阶段完成必要的快速初始化；等待用户操作的 Dialog 等交互由事件流程触发，保持插件加载和事件分发顺畅。
- 采用日志初始化函数与 `tracing` 记录关键阶段、设备通信状态和错误；日志遵循凭据与用户数据保护规则。
- 异步运行模型由 WIT/Component Model 与绑定工具链提供；禁止引入 `#[tokio::main]`、`tokio::spawn` 或自建 Tokio executor 替代插件运行模型。
- UI 使用元素 Builder、事件 ID 与 Host 渲染接口；具体参数按 V4 WIT 实现。
- 使用第三方 crate 时验证 WASI 兼容性；优先选用适配目标平台的纯 Rust 依赖，单独检查原生库、进程、线程及系统接口依赖。

#### 阅读 V2 示例时的迁移对照

| 教程中的 V2 写法 | 本项目采用的 V4 写法 |
| --- | --- |
| `psys-world` / `astrobox:psys-host` | `psys-world-v4` / `astrobox:psys-host-v4` |
| 同步 `fn on_load()` | 按生成绑定实现异步 `on-load` |
| `future<T>` / `FutureReader<T>` | WIT `async func` 与对应的 Rust async 实现 |
| `wit_future::new()` + writer/reader + `wit_bindgen::spawn` 回传结果 | 直接 await 调用并按 async Guest 签名返回结果 |
| 同步初始化中的 `wit_bindgen::block_on` | 异步 `on-load` 中正常 `.await` |

阅读教程时保留其组件边界、工程组织和事件设计思路；编写、审查及验证代码统一采用 V4 契约。

### JavaScript

- 从[官方 V4 JavaScript 模板][js-template]学习；模板依赖官方维护的 ComponentizeJS 补丁分支 `astrobox/async-func-support`。
- 文档核对时，上游 ComponentizeJS 至 0.22 尚未实现 `async func`；不能随意把模板依赖换成上游版本。升级前重新验证 async 支持和实际构建。
- 构建环境需要 Node.js、Rust，以及 `wasm32-wasip1` target；npm 安装 Git 依赖时会构建 splicer。这是**构建期**依赖，不代表插件运行时拥有 Node.js 能力。
- **关键差异：声明为 WIT `async func` 的宿主接口，在该 JS 工具链中也按同步方式调用。** 例如 `getRenderSize()` 直接返回结果，调用期间阻塞到宿主返回；不要设计依赖其返回 Promise 的并发逻辑。示例中出现 `await` 也不能证明返回的是 Promise。
- 插件导出对象名对应接口名：`lifecycle`、`event`；导出函数可以是 `async`，引擎会在返回前结算 Promise。
- WIT enum 在 JS 中是字符串，例如 `new Element('div', null)`；不要使用不存在的 `ElementType.DIV` 导出。
- WIT record、variant、option 的 JS 表示以实际绑定为准；例如安装源示例使用 `{ tag: 'file', val: 'downloads/app.rpk' }`，不要自行转换为任意 JSON 结构。
- 不假设 Node 内置模块、DOM、浏览器 `fetch` 或 npm 依赖可直接在 WASM 运行时使用；逐一核实工具链和 WASI/Host API 能力。
- 官方模板的 WASM 约 14 MB，内嵌 SpiderMonkey；这只是参考值，不是本项目的包体门槛。[JS 上手][javascript]

### Python

- 官方教程使用 `componentize-py==0.25.1`；语言选择页说明 async 支持要求 0.25+。
- 入口模块按 WIT 导出接口名提供实现类 **`Lifecycle`** 与 **`Event`**，不能随意改名；按生成绑定实现 async 方法和宿主调用。
- 运行时导入应在模块顶层解析完成，必要时显式 `from x import y`，不要假定构建后还能动态发现子模块。
- 官方模板的 WASM 约 20 MB，内嵌 CPython；首次加载会预编译并缓存，不能据此省略冷启动测试。[Python 上手][python]

## 4. WIT 与依赖管理

- 官方模板把 [AstroBox-Plugin-WIT][wit-repo]作为 `wit/` 子模块使用。初始化已有模板时检查子模块是否完整，后续提交锁定实际使用的子模块 commit。
- 本次接口补充核对使用官方 JS 模板当时引用的 WIT commit `61d8681e2708f95fb8c403aeb8147d416dbfe03a`，见[宿主接口][host-wit-v4]与[插件导出][plugin-wit-v4]。该提交作为本指南的资料快照，后续工程记录实际采用的 commit。
- 修改 API Level、WASI 版本、绑定生成器或 WIT 时，必须同步审查清单、构建 world、所有 import/export 与测试；不能只改一个字段。
- 不手改自动生成绑定来掩盖版本不匹配，不手改编译产物替代源码修复。
- 本工程使用 Rust 1.100.0、wit-bindgen 0.63.0；官方 WIT 固定提交以 vendored 快照保存，来源见 [WIT 记录](wit/PROVENANCE.md)。Cargo 构建包装器把工具缓存与临时文件隔离在项目 .tools 目录。

## 5. 插件清单与分发

以下是 **V4 清单示意**，用于解释字段，不是已经选定的项目身份；名称、作者、版本、主页和权限必须按实际项目确定：

```json
{
  "name": "Example Plugin",
  "icon": "icon.png",
  "version": "0.1.0",
  "description": "替换为实际插件简介",
  "author": "替换为实际作者",
  "website": "https://example.com/replace-me",
  "entry": "plugin.wasm",
  "wasi_version": 3,
  "api_level": 4,
  "permissions": []
}
```

- 清单文件名为 [manifest.json][js-manifest]，必须是合法 JSON；入口、图标与资源路径必须能在分发包中找到，不能留下模板占位值。
- 字段结构参考[Manifest 文档][manifest]，V4 版本组合已用[官方 JS 模板清单][js-manifest]交叉核对。不要复制 Manifest 页完整示例中的旧 `wasi_version: 1`、`api_level: 1` 作为 V4 配置。
- 对**需要权限的能力**，在 `permissions` 中声明最小集合；**声明权限不等于用户已经授权**，还要按具体接口处理首次询问、拒绝及撤销。并非所有接口都需要权限，例如 V4 WebSocket 不要求额外权限；不能自行添加未核实的权限名。
- `additional_files` 用于官方插件源分发时枚举附加文件、参与下载及体积统计；不能误以为填入字段就会自动打包这些文件。仅通过 `.abp` 分发时，文档说明无需为此填写。
- `.abp` 是 ZIP 容器。按官方 JS 模板的打包方式，压缩的是产物目录的内容，包根包含清单与入口，而不是再套一层产物目录。[官方构建脚本][js-build]
- 分发的是可加载的 WebAssembly **Component**，不是任意 core wasm，也不是宿主生成的预编译缓存。
- 打包前实际检查所有附加资源。官方 JS 构建脚本只复制入口、清单及可找到的图标，新增资源需要相应调整构建脚本。
- 不提交密钥、令牌、签名私钥、用户数据或预编译缓存；是否提交可分发产物按后续仓库发布策略决定。

## 6. 生命周期、UI 与错误处理

- 按所选 WIT 实现宿主要求的导出。V4 JS 入门展示 `lifecycle.onLoad` 与 `event.onEvent`、`onUiEvent`、`onUiRender`、`onCardRender`，签名以实际绑定为准。
- UI 通过 Host API 的 `Element`、`render`、渲染尺寸及事件机制构建，不是向任意浏览器 DOM 注入 HTML；根据容器尺寸适配，不能把设备端 Vela 的设计像素或屏幕坐标当作 AstroBox 页面布局规则。
- 在 `on-ui-render(element-id)` 中向相同 ID 提交 UI 树，用元素的 `.on(event, id)` 绑定事件。渲染尺寸为容器 CSS px，未挂载或未知时可能返回 0，不能直接当作设备物理分辨率。[宿主接口][host-wit-v4]
- 把页面渲染、业务状态、设备通信和安装状态分开；渲染不自动重复发起安装、授权或有副作用的请求。
- V4 错误使用 `result<_, string>`，保留原因并给用户可理解的提示；JS 绑定的失败可能抛异常，应按绑定约定捕获。不要依赖英文错误文案做业务分支。[Level 4 变更][v4-new] [MASS][mass]
- 所有等待、重试、断线恢复都要有明确的状态与退出路径；操作超时不等于设备操作已取消，不能假报成功或无条件重放副作用。
- 本次核对提交的 `lifecycle` 只有 `on-load`，没有 `on-unload` 导出；不得虚构 `onUnload` 清理回调，资源关闭和任务终止按实际 API 设计。[插件导出][plugin-wit-v4]

## 7. 设备交互与安装

### 互联通信

- 如果需要对接设备端游戏，先确定设备、快应用包名和设备端实现，再设计通信；不得宣称当前项目已经具备互联能力。
- V4 WIT 已确认 `interconnect.send-qaic-message(device-addr, pkg-name, data: string)` 用于发送；先调用 `register.register-interconnect-recv(addr, pkg-name)`，才接收 `interconnect-message`。两个接口均为 `async func`，返回 `result<_, string>`，不是旧 `future<result>`。[宿主接口][host-wit-v4] [插件事件][plugin-wit-v4]
- 具体 V4 权限名称及事件 payload 的宿主编码仍需在实现时核实，不能把[旧版 Interconnect 文档][interconnect-v2]当成全部 V4 行为的证明。
- 协议设计应包含版本、请求 ID、消息类型、边界校验、超时、错误与幂等规则；收到的文本、JSON、路径、设备地址和包名都视为不可信输入。
- 没有用户明确要求，不安装、卸载、刷写设备，不清除游戏存档，不修改资产或绕过设备授权。

### 首选宿主安装队列

- 通常优先 `queue.enqueue-install`，复用宿主队列、进度、重试、断线恢复及压缩包处理；需要声明 `queue`，创建安装任务首次会请求授权。
- **入队成功不代表安装成功。** 保存返回的 task ID，处理 `queue.task` 的后续状态；宿主消息带 `host: true`，不能只凭同名插件广播事件认定宿主已完成安装。
- `file` 来源必须是插件目录内的相对路径。宿主会**移走**文件，入队后不要继续读取、写入或删除它；错误时宿主会尝试放回，仍需检查实际结果。
- 大包优先文件来源，避免把整个包加载成 `bytes`。V4 队列文档列出插件内存上限 **256 MiB**、单包上限 **1 GiB**，不能把旧 V2 运行环境页的 128 MiB 当作 V4 上限。
- `cancel-task` 只能取消本插件创建且还没开始安装的任务；不能承诺可以中断正在安装的任务。[安装队列][queue]

### MASS 仅用于明确需要的直传

- `mass.send` 是原始载荷通道，没有队列的完整包装；`.rpk` 是快应用载荷，`.mwz` 等压缩包不能不解包直接发送。
- 直传只支持相应的小米协议设备，同一设备不能同时安装；宿主不校验载荷内容，插件应先验证来源、类型、完整性与设备适配。
- 快应用直传应提供真实包名，不依赖默认占位包名 `a.b.c`。
- `mass` 与 `mass.firmware` 权限不同；固件每次都需用户确认，绝不把固件权限作为普通快应用安装的默认权限。[MASS][mass]

### 接管资源安装是可选扩展

- 只有业务确需接管资源下载/安装按钮时，才使用 `psys-world-v4-resource-install` 并实现 `resource-install.on-install`。
- `ext.pluginInstall` 属于资源 manifest，不是插件自身清单字段；其中的插件名必须匹配插件清单的 `name`，`data` 为公开 JSON，UTF-8 序列化后不超过 16 KiB，不能放密钥。
- 明确处理 `queued`、`redeem`、`pass`、`failed`；handler 时限为 3 分钟，长传输应入队后返回，不能让页面无限等待。
- 配置无效可能被整体忽略并回到普通下载；`fallback` 默认 `block`，使用 `download` 会允许走常规下载，不能把它当成强制核销保障。[资源接管安装][resource-install]

## 8. 可选能力、权限与数据安全

仅在业务需要时引入下列能力，不预设本项目需要网络登录、账号识别或通知：

| 能力 | 已核实的边界 |
| --- | --- |
| HTTP 服务器 | `http-server` 只允许回环地址；绑定所有网卡还需 `http-server.lan`。必须使用 `psys-world-v4-http` 并实现 HTTP handler；端口 0 由系统分配，请求体上限 16 MiB。停用或重载插件会释放监听。[文档][http-server] |
| WebSocket | 无额外权限，但要确认目标宿主实际提供该导入。连接时选择推送或拉取，之后不能切换。**JS 使用推送，不调用会阻塞整个插件的 `receive` 等待循环**；检查宿主消息 `host: true`，及时返回事件处理，显式关闭连接。[文档][websocket] |
| 应用内浏览器 | 声明 `browser`，每次 `open` 都要授权。OAuth 拦截仅按字符串前缀匹配，业务仍需严格校验回调来源及 state。Android 的 cookie 与 `ephemeral` 隔离能力有限，不能承诺真正独立的无痕会话。[文档][browser] |
| 设备通知/实时活动 | 声明 `notification`。接口成功只代表加入传输队列，不代表手表已经显示；显示样式及超级岛支持必须真机确认，不能当作电脑/手机系统通知 API。[文档][notification] |
| 宿主标识/账号资料 | 分别使用 `os.device-id`、`account.profile` 权限。宿主标识是 AstroBox 安装标识，不是硬件序列号或手环地址；账号接口不返回 token/Cookie，未登录和绑定 `unavailable` 不能误判为已授权或已绑定。[文档][identity] |

- 不记录或提交凭据、OAuth 完整回调 URL、Cookie、用户输入的密码和不必要的设备标识；对外上传用户数据前说明用途并取得授权。
- 本次核对的 V4 Host WIT 没有独立的 HTTP 客户端、filesystem 或 storage 接口；这**不代表 WASI 没有网络或文件能力**。HTTP 客户端、路径映射、插件数据持久化须按目标语言/WASI/宿主核实，不自行补造 `fs`、`fetch`、`localStorage` 等运行时接口。

## 9. 后续构建与验证流程

下列为**对应官方模板的构建命令参考**，供后续选型与工程初始化完成后使用：

```text
# JavaScript 官方模板
# 准备 Node.js、Rust、wasm32-wasip1，并初始化 wit 子模块后：
npm install
npm run build
npm run package

# Python 官方模板
python scripts/build_dist.py --package
```

- 官方 JS 打包脚本调用系统 `zip`；当前开发环境为 Windows，后续运行前要检查是否可用，不能把文档里的 Unix 工具假定为已安装。若改为跨平台打包，保持包内布局并实际验证。
- 官方 JS 脚本会清空模板的产物目录；更改脚本或运行前确认输出路径，不能把源文件、签名材料或用户数据放进可清理目录。
- 构建完成后核对清单 JSON、必填项、API/WASI/world 一致性、组件导入导出、入口/图标/资源存在性及 `.abp` 内部布局。
- 初始化工程后再按实际语言补上可重复执行的测试命令；不能把未创建的 npm script、测试套件或构建报告写成已存在。
- 逻辑测试至少覆盖：载荷解析与边界、错误、超时、重试幂等、权限拒绝/撤销、重复事件、断线与重连；安装功能还要覆盖排队成功与设备安装成功的区别。
- 宿主验证覆盖：实际客户端版本、冷/热加载、启用/停用/重载、页面尺寸、UI 事件、日志、权限流程及资源释放。实际启用或安装前取得对应授权。
- 分别确认插件已启用、已加载、已初始化就绪；“安装成功”不等于可用。官方报错参考说明非 Tauri 的 Web/WASM 环境不支持插件安装，浏览器预览不能替代原生宿主验证。[插件报错参考][plugin-errors]
- 本次未核实 V4 通用开发服务器、热更新命令或统一测试命令，不得虚构这些入口；重载验证不代表已配置自动热更新。
- 设备功能必须记录设备型号、固件、连接方式和实际步骤；主机单测、构建成功、插件加载成功都不能替代真机通信、安装或显示验证。
- 验收说明分开记录“静态核对”“构建通过”“宿主通过”“真机通过”；未执行或未验证的部分明确写出，禁止继承旧项目的测试数量和真机结论。

## 10. 后续代理的工作方式

- 开始实现前先阅读本文件和实际项目配置，围绕固定的 **API Level 4 / WASI 3** 核对语言、WIT commit、权限、构建与测试入口；生成和审查代码时保持 V4 目标一致。
- 先实现最小闭环，再增加所需能力；不要一次性申请全部权限或引入与当前任务无关的服务。
- 当前功能范围为四页 UI Demo、本地演示快照、备份保留与 JSON 导出。真实设备通信、设备存档恢复和新业务能力由后续任务单独定义。
- 代码改动保持可审查，补充相关验证；修改依赖、升级 WIT 或改变分发格式时更新本指南和发布说明。
- 提交、推送、发布、真实设备操作与凭据使用遵循用户的明确任务边界；不把阅读文档或生成指南当作这些操作的授权。

## 11. 本工程的实施与交付约定

- 源码使用 Rust，清单始终为 API Level 4 / WASI 3。当前构建与测试命令见 [README](README.md)。
- UI 根据实际容器宽度切换手机、平板、电脑布局。原生 Host 的多列布局采用公开 Flex Builder 组合，语义 Grid 在适配前进行可测试转换；ScrollArea 设置明确视口高度。
- 所有演示数据显式标记 mock，部门任务标记 demo-extension；累计承伤与结束伤势分开保存，原游戏作为只读参考。
- 默认保留 5 份备份，可设 1–50 份；外部导出副本由玩家独立保管。本地写入成功后再提交保留清理和恢复状态。
- 导出走 V4 dialog 系统保存会话，使用分片写入、finish/abort 与 SHA-256 完整性检查；格式见 [备份协议](docs/backup-format.md)。
- 新代码采用 [MIT](LICENSE)，第三方许可随源码和发布包保留，参考 [声明](THIRD_PARTY_NOTICES.md)。
- 唯一推送目标为用户仓库 taskschd1145/deltaforce-watch-astrobox-plugin；每次代码提交使用分支及对应 PR，gh CLI 创建后保留待用户合并。
- 作者仓库根索引写 dist，发布目录包含清单声明的全部文件。官方登记流程记录在 [发布文档](docs/publishing.md)，上游提交等待用户明确指示。

## 官方资料索引

下列为本指南整理依据；Rust 开发优先阅读用户指定的上手教程，再结合当前任务所选 API Level 的文档和锁定版本源码核对实现。

- **重点必读：[Rust 插件开发上手教程](https://abox.run/docs/plugin-development/v2/getstarted/rust)**
- [官方 Rust 模板](https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Template-Rust)
- [插件开发入口 / V4 概览][overview]
- [Level 4 变更与兼容性][v4-new]
- [语言选择与 JS async 语义][language]
- [JavaScript 上手][javascript]、[Python 上手][python]
- [官方 JavaScript 模板][js-template]、[模板清单][js-manifest]、[模板构建脚本][js-build]
- [WIT 仓库][wit-repo]、[世界定义][wit-world]、[本次核对的 V4 宿主接口][host-wit-v4]与[插件导出][plugin-wit-v4]
- [Manifest 通用字段（V2 文档，示例版本号不能照抄）][manifest]、[API Level 对应表][api-level]、[插件报错参考][plugin-errors]
- [安装队列][queue]、[MASS][mass]、[资源接管安装][resource-install]
- [Interconnect 旧版概念参考（实现 V4 前重新核对签名）][interconnect-v2]
- [HTTP 服务器][http-server]、[WebSocket][websocket]、[应用内浏览器][browser]
- [设备通知与实时活动][notification]、[宿主标识与账号资料][identity]

[overview]: https://abox.run/docs/plugin-development/v4
[v4-new]: https://abox.run/docs/plugin-development/v4/whats-new
[language]: https://abox.run/docs/plugin-development/v4/getstarted/language
[javascript]: https://abox.run/docs/plugin-development/v4/getstarted/javascript
[python]: https://abox.run/docs/plugin-development/v4/getstarted/python
[js-template]: https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Template-JS
[js-manifest]: https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Template-JS/blob/main/manifest.json
[js-build]: https://github.com/AstralSightStudios/AstroBox-NG-Plugin-Template-JS/blob/main/scripts/build.mjs
[wit-repo]: https://github.com/AstralSightStudios/AstroBox-Plugin-WIT
[wit-world]: https://github.com/AstralSightStudios/AstroBox-Plugin-WIT/blob/main/main.wit
[manifest]: https://abox.run/docs/plugin-development/v2/general/manifest
[queue]: https://abox.run/docs/plugin-development/v4/host-api/queue
[mass]: https://abox.run/docs/plugin-development/v4/host-api/mass
[resource-install]: https://abox.run/docs/plugin-development/v4/resource-install
[interconnect-v2]: https://abox.run/docs/plugin-development/v2/host-api/interconnect
[http-server]: https://abox.run/docs/plugin-development/v4/host-api/http-server
[websocket]: https://abox.run/docs/plugin-development/v4/host-api/websocket
[browser]: https://abox.run/docs/plugin-development/v4/host-api/browser
[notification]: https://abox.run/docs/plugin-development/v4/host-api/notification
[identity]: https://abox.run/docs/plugin-development/v4/host-api/identity
[api-level]: https://abox.run/docs/plugin-development/v2/general/apilevel
[host-wit-v4]: https://github.com/AstralSightStudios/AstroBox-Plugin-WIT/blob/61d8681e2708f95fb8c403aeb8147d416dbfe03a/deps/astrobox-psys-host-v4.wit
[plugin-wit-v4]: https://github.com/AstralSightStudios/AstroBox-Plugin-WIT/blob/61d8681e2708f95fb8c403aeb8147d416dbfe03a/deps/astrobox-psys-plugin-v4.wit
[plugin-errors]: https://abox.run/docs/usage/errors/plugin-errors
