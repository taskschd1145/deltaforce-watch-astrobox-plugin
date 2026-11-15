//! Deterministic UI demonstration fixtures. Every value in this module is simulated.
use crate::contracts::*;

fn item(
    id: &str,
    name: &str,
    quality: &str,
    category: &str,
    quantity: u32,
    unit_value: u64,
    slots: u32,
    locked: bool,
    durability: Option<(u32, u32)>,
    compartment: &str,
) -> Item {
    Item {
        id: id.into(),
        name: name.into(),
        quality: quality.into(),
        category: category.into(),
        quantity,
        unit_value,
        slots,
        locked,
        durability: durability.map(|v| v.0),
        max_durability: durability.map(|v| v.1),
        compartment: compartment.into(),
    }
}

fn inventory() -> Vec<Item> {
    vec![
        item(
            "inv-m4a1",
            "M4A1 突击步枪",
            "紫",
            "装备",
            1,
            186_000,
            6,
            true,
            Some((87, 100)),
            "warehouse",
        ),
        item(
            "inv-akm",
            "AKM 突击步枪",
            "蓝",
            "装备",
            1,
            72_000,
            6,
            false,
            Some((64, 100)),
            "warehouse",
        ),
        item(
            "inv-mp5",
            "MP5 冲锋枪",
            "绿",
            "装备",
            1,
            38_000,
            4,
            false,
            Some((95, 100)),
            "warehouse",
        ),
        item(
            "inv-helmet",
            "DICH 战术头盔",
            "紫",
            "装备",
            1,
            96_000,
            4,
            true,
            Some((42, 60)),
            "warehouse",
        ),
        item(
            "inv-armor",
            "重型防弹衣",
            "金",
            "装备",
            1,
            245_000,
            9,
            true,
            Some((91, 120)),
            "warehouse",
        ),
        item(
            "inv-pack",
            "突击背包",
            "蓝",
            "装备",
            1,
            62_000,
            6,
            false,
            Some((100, 100)),
            "warehouse",
        ),
        item(
            "inv-m855",
            "5.56×45mm M855 弹药",
            "蓝",
            "弹药",
            180,
            620,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-m995",
            "5.56×45mm M995 弹药",
            "金",
            "弹药",
            60,
            2_100,
            1,
            true,
            None,
            "warehouse",
        ),
        item(
            "inv-762",
            "7.62×39mm PS 弹药",
            "绿",
            "弹药",
            120,
            260,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-9mm",
            "9×19mm 普通弹药",
            "白",
            "弹药",
            90,
            95,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-bandage",
            "无菌绷带",
            "白",
            "药品",
            6,
            1_800,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-medkit",
            "战地急救包",
            "蓝",
            "药品",
            3,
            18_500,
            2,
            false,
            Some((240, 300)),
            "warehouse",
        ),
        item(
            "inv-surgery",
            "手术工具包",
            "紫",
            "药品",
            1,
            46_000,
            2,
            true,
            Some((4, 5)),
            "warehouse",
        ),
        item(
            "inv-painkiller",
            "缓释止痛药",
            "绿",
            "药品",
            4,
            7_500,
            1,
            false,
            Some((4, 4)),
            "warehouse",
        ),
        item(
            "inv-gold",
            "金条",
            "金",
            "贵重品",
            2,
            168_000,
            1,
            true,
            None,
            "warehouse",
        ),
        item(
            "inv-red",
            "非洲之心",
            "红",
            "贵重品",
            1,
            1_280_000,
            1,
            true,
            None,
            "warehouse",
        ),
        item(
            "inv-drive",
            "加密硬盘",
            "紫",
            "贵重品",
            2,
            58_000,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-watch",
            "机械腕表",
            "蓝",
            "贵重品",
            1,
            24_000,
            1,
            false,
            None,
            "warehouse",
        ),
        item(
            "inv-key-dam",
            "零号大坝变电站房卡",
            "紫",
            "房卡",
            1,
            89_000,
            1,
            true,
            Some((7, 10)),
            "warehouse",
        ),
        item(
            "inv-key-hotel",
            "长弓溪谷酒店房卡",
            "金",
            "房卡",
            1,
            216_000,
            1,
            false,
            Some((3, 10)),
            "warehouse",
        ),
    ]
}

fn damage(seed: u32, injured: bool) -> Vec<DamagePart> {
    ["head", "chest", "abdomen", "arm", "leg"]
        .iter()
        .enumerate()
        .map(|(i, part)| DamagePart {
            part: (*part).into(),
            // Cumulative damage includes healing; it is deliberately independent of final injury.
            damage_taken: Some(seed + i as u32 * 31),
            injury_at_end: Some(if injured {
                (seed + i as u32 * 7) % 71
            } else {
                0
            }),
        })
        .collect()
}

fn matches() -> Vec<MatchRecord> {
    let operations = [
        (
            "op-008",
            "extracted",
            "航天基地",
            "机密",
            1_794_708_000_000,
            1_420,
            6,
        ),
        (
            "op-007",
            "killed",
            "零号大坝",
            "普通",
            1_794_703_200_000,
            937,
            2,
        ),
        (
            "op-006",
            "extracted",
            "长弓溪谷",
            "机密",
            1_794_698_400_000,
            1_608,
            4,
        ),
        (
            "op-005",
            "timeout",
            "巴克什",
            "机密",
            1_794_693_600_000,
            1_800,
            3,
        ),
        (
            "op-004",
            "extracted",
            "零号大坝",
            "普通",
            1_794_621_600_000,
            1_100,
            5,
        ),
        (
            "op-003",
            "killed",
            "航天基地",
            "机密",
            1_794_616_800_000,
            620,
            1,
        ),
        (
            "op-002",
            "extracted",
            "巴克什",
            "机密",
            1_794_535_200_000,
            1_560,
            7,
        ),
        (
            "op-001",
            "extracted",
            "长弓溪谷",
            "普通",
            1_794_530_400_000,
            1_260,
            3,
        ),
    ];
    let mut records = Vec::new();
    for (index, (id, outcome, map, difficulty, started_at, duration, kills)) in
        operations.into_iter().enumerate()
    {
        let failed = outcome != "extracted";
        let mut loot = vec![item(
            &format!("{id}-safe"),
            "加密硬盘",
            "紫",
            "贵重品",
            1 + index as u32 % 2,
            58_000,
            1,
            false,
            None,
            "safe",
        )];
        if !failed {
            loot.push(item(
                &format!("{id}-gold"),
                "金条",
                "金",
                "贵重品",
                1 + index as u32 % 3,
                168_000,
                1,
                false,
                None,
                "backpack",
            ));
            loot.push(item(
                &format!("{id}-med"),
                "战地急救包",
                "蓝",
                "药品",
                2,
                18_500,
                2,
                false,
                Some((300, 300)),
                "backpack",
            ));
        }
        let loot_value = loot
            .iter()
            .map(|v| v.unit_value * u64::from(v.quantity))
            .sum();
        records.push(MatchRecord {
            id: id.into(),
            mode: "operations".into(),
            outcome: outcome.into(),
            map_name: map.into(),
            difficulty: difficulty.into(),
            started_at: Some(started_at),
            duration_seconds: duration,
            kills,
            loot_value,
            score: 640 + index as u32 * 117,
            xp_gain: 1_500 + index as u32 * 330,
            deaths: u32::from(failed),
            captures: 0,
            revives: index as u32 % 3,
            loot,
            damage: damage(22 + index as u32 * 17, failed),
            source: "mock".into(),
        });
    }
    for (index, (id, outcome, map, started_at)) in [
        ("bf-004", "victory", "临界点", 1_794_706_200_000),
        ("bf-003", "defeat", "攀升", 1_794_700_800_000),
        ("bf-002", "draw", "烬区", 1_794_619_200_000),
        ("bf-001", "practice", "训练场", 1_794_532_800_000),
    ]
    .into_iter()
    .enumerate()
    {
        records.push(MatchRecord {
            id: id.into(),
            mode: "battlefield".into(),
            outcome: outcome.into(),
            map_name: map.into(),
            difficulty: if outcome == "practice" {
                "训练"
            } else {
                "标准"
            }
            .into(),
            started_at: Some(started_at),
            duration_seconds: 1_020 + index as u32 * 120,
            kills: 24 - index as u32 * 4,
            loot_value: 0,
            score: 8_420 - index as u32 * 730,
            xp_gain: 2_350 - index as u32 * 180,
            deaths: 6 + index as u32,
            captures: 4 - index as u32,
            revives: 9 - index as u32 * 2,
            loot: vec![],
            damage: damage(146 + index as u32 * 55, true),
            source: "mock".into(),
        });
    }
    records.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    records
}

/// Stable mock data for the complete UI demo. All UI data is simulated.
pub fn initial_data() -> PersistedData {
    let inventory = inventory();
    let snapshot = Snapshot {
        schema_version: STORE_VERSION,
        source: "mock".into(),
        captured_at: DEMO_CAPTURED_AT,
        source_game_version: "1.2.2.4".into(),
        player: Player {
            name: "三角洲演示干员".into(),
            level: 38,
            funds: 3_286_500,
            warehouse_capacity: 160,
            warehouse_used: inventory.iter().map(|v| v.slots).sum(),
            completed_runs: 128,
        },
        matches: matches(),
        inventory,
        facilities: vec![
            Facility {
                id: "warehouse".into(),
                name: "仓库".into(),
                level: 4,
                max_level: 6,
                description: "模拟设施 · 整理装备与物资".into(),
                effect: "仓库容量 160 格（演示）".into(),
            },
            Facility {
                id: "training-center".into(),
                name: "训练中心".into(),
                level: 3,
                max_level: 5,
                description: "模拟设施 · 干员训练与能力成长".into(),
                effect: "训练效率 +15%（演示）".into(),
            },
        ],
        missions: [
            (
                "combat",
                "战斗部门",
                "前线突击",
                7,
                10,
                "击败 10 名敌人",
                "80,000 金币",
            ),
            (
                "medical",
                "医疗部门",
                "战地援助",
                3,
                3,
                "完成 3 次队友救援",
                "战地急救包 ×2",
            ),
            (
                "logistics",
                "后勤部门",
                "物资回收",
                2,
                5,
                "带回 5 件蓝色及以上物资",
                "60,000 金币",
            ),
            (
                "research",
                "研发部门",
                "设备采样",
                1,
                4,
                "提交 4 份演示设备样本",
                "120,000 金币",
            ),
            (
                "intel",
                "情报部门",
                "区域侦察",
                4,
                4,
                "完成 4 处区域侦察",
                "加密硬盘 ×1",
            ),
            (
                "trade",
                "贸易部门",
                "安全运输",
                1,
                3,
                "完成 3 次成功撤离",
                "90,000 金币",
            ),
        ]
        .into_iter()
        .map(
            |(id, department, title, progress, target, description, reward)| Mission {
                id: format!("mission-{id}"),
                department: department.into(),
                title: title.into(),
                description: format!("Demo 扩展任务（demo-extension）· {description}"),
                progress,
                target,
                reward: format!("{reward}（模拟）"),
                source: "demo-extension".into(),
            },
        )
        .collect(),
    };
    let mut backups = Vec::new();
    for (id, created_at, funds, red_quantity) in [
        ("demo-backup-003", 1_794_717_000_000, 3_120_000, 1),
        ("demo-backup-002", 1_794_714_000_000, 2_840_000, 2),
        ("demo-backup-001", 1_794_711_000_000, 2_430_000, 3),
    ] {
        let mut older = snapshot.clone();
        older.captured_at = created_at;
        older.player.funds = funds;
        if let Some(item) = older.inventory.iter_mut().find(|v| v.id == "inv-red") {
            item.quantity = red_quantity;
        }
        backups.push(BackupRecord {
            id: id.into(),
            created_at,
            source: "mock".into(),
            device_name: "演示手表 · 模拟设备".into(),
            snapshot: older,
        });
    }
    PersistedData {
        schema_version: STORE_VERSION,
        settings: PluginSettings {
            auto_backup_on_launch: false,
            retention_count: 5,
        },
        snapshot,
        backups,
    }
}
