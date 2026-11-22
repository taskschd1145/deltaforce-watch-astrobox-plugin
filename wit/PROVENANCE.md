# WIT 接口来源快照

- 上游：https://github.com/AstralSightStudios/AstroBox-Plugin-WIT
- 固定提交：`61d8681e2708f95fb8c403aeb8147d416dbfe03a`
- 许可：MIT，见 [LICENSE](LICENSE)。
- 本项目将该提交的世界定义与四份依赖 WIT 原样保存在本目录，便于单仓库离线构建与审阅；[Rust 入口](../src/runtime.rs)使用 `psys-world-v4`。

[世界定义](main.wit)还包含 Level 2/3 世界，用于保留官方文件完整内容。插件清单、绑定选择与实际组件均以 Level 4 为目标。
