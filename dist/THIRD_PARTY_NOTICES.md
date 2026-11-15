# 第三方资料与许可

## 本插件

新编写的插件代码、界面组件、几何图标和演示数据由 taskschd1145 以 MIT 许可发布，见 LICENSE。

本项目为“三角洲行动 Vela 手表版”的配套爱好者工具。界面示例使用明确标注的模拟数据。游戏名称、地图名称等标识的相关权利归相应权利人所有。

## AstroBox Plugin WIT

- 仓库：https://github.com/AstralSightStudios/AstroBox-Plugin-WIT
- 锁定提交：61d8681e2708f95fb8c403aeb8147d416dbfe03a
- 许可：MIT，Copyright (c) 2025 Searchstars。
- 完整许可随源码子模块及发布包 licenses/AstroBox-Plugin-WIT-MIT.txt 保存。

插件通过公开 WIT 接口独立实现，与 AstroBox 宿主协作。AstroBox 主程序代码保持为只读接口参考。

## Rust 依赖

精确版本与校验值记录在 Cargo.lock。使用的依赖包括：

- serde / serde_json：MIT OR Apache-2.0。
- sha2 及 RustCrypto 基础依赖：MIT OR Apache-2.0。
- chrono：MIT OR Apache-2.0。
- wit-bindgen：Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT。

构建与测试环境可包含更多传递依赖；发布前以 Cargo 元数据和锁文件核对其许可证。

## 原游戏视觉参考

只读参考仓库：https://github.com/taskschd1145/deltaforce-watch-vela

参考范围为深灰绿配色、低圆角卡片、物品品质颜色、字段含义与信息层级。插件界面、状态机、快照格式和导出流程独立编写；标志和人体图以几何形状重新绘制。演示物资的价格、统计与任务属于界面示例。

## 文档

AstroBox 文档链接用于接口与分发流程引用：https://abox.run/docs/plugin-development/ 。本仓库的开发说明采用独立整理与来源链接。
