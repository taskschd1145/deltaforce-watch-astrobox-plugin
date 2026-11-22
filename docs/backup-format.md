# 演示备份导出格式 v1

## 使用

在设置的本地快照库中选择一份备份，点击“导出”，由系统保存窗口选择文件名和位置。导出文件为 UTF-8 JSON，默认名称类似 `deltaforce-sync-demo-20261115-070000-00000001.json`；文件名中的时间使用 UTC，内容保存明确时间字段。

文件保留完整快照，可复制到其他目录或个人文件存储中独立保管。插件内部的保留份数策略只管理插件数据目录中的备份。

## 结构

```json
{
  "format": "deltaforce-sync-backup",
  "formatVersion": 1,
  "pluginVersion": "0.1.0",
  "exportedAt": "2026-11-15T07:00:00.000Z",
  "backup": {
    "id": "backup-example",
    "createdAt": 1794726000000,
    "source": "mock",
    "sourceGameVersion": "1.2.2.4",
    "deviceName": "演示手表 · 模拟设备",
    "snapshot": "完整 Snapshot 对象，以实际导出内容为准"
  },
  "integrity": {
    "algorithm": "sha256",
    "digest": "64位小写十六进制摘要"
  }
}
```

上例用于说明层次；真实 `snapshot` 是包含玩家、对局、库存、设施和任务的完整对象。`createdAt` 及快照时间为 Unix 毫秒，`exportedAt` 为 RFC3339 UTC。

## 确定性散列

[导出实现](../src/export.rs)采用以下明确约定：

1. 将完整 envelope 转为 JSON 值，移除根字段 `integrity`。
2. 递归按对象键排序，数组保持原顺序。
3. 序列化为紧凑 JSON：UTF-8，无 BOM，无缩进，无尾部换行。
4. 对上述字节计算 SHA-256，编码为 64 位小写十六进制，写入 `integrity.digest`。
5. 最终文件使用可读缩进并添加尾部换行；这些展示空白位于散列计算之外。

验证器检查唯一对象键、完整字段形状、版本、模拟来源、数值边界、时间关系及摘要，再返回完整 BackupRecord。整数应以支持其完整精度的解析器处理。

SHA-256 用于检测内容变化与损坏；文件来源的可信度由玩家与可信分发渠道确认。完整快照包含玩家资料及物资信息，分享前请按自己的隐私需求检查内容。

## 保存会话

V4 适配器通过 `dialog.save-file-start` 获取系统保存会话，按 64 KiB 分片写入，调用 finish 完成。异常路径调用 abort；原备份与当前快照继续保留。保存窗口取消与宿主错误显示相应提示，导出状态统一释放。

格式 v1 当前描述本版演示快照；后续真实游戏备份格式会通过新的版本与能力标记明确兼容范围。
