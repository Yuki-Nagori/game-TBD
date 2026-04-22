//! 核心模块
//!
//! 游戏核心逻辑：ECS 组件、系统、事件定义

use bevy::prelude::*;

/// 游戏内时间资源
#[derive(Resource, Debug)]
#[allow(dead_code)]
pub struct GameTime {
    /// 当前年份
    pub year: i32,
    /// 当前月份（1-12）
    pub month: u8,
    /// 当前日期（1-30）
    pub day: u8,
    /// 当前小时（0.0 - 24.0）
    pub hour: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            year: 1573, // 万历元年
            month: 1,
            day: 1,
            hour: 6.0,
        }
    }
}

#[allow(dead_code)]
impl GameTime {
    /// 推进时间（游戏内小时）
    pub fn advance(&mut self, hours: f32) {
        self.hour += hours;

        while self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
        }

        // 简化：每月30天
        while self.day > 30 {
            self.day -= 30;
            self.month += 1;
        }

        while self.month > 12 {
            self.month -= 12;
            self.year += 1;
        }
    }
}

impl std::fmt::Display for GameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}年{:02}月{:02}日 {:02}:00",
            self.year, self.month, self.day, self.hour as i32
        )
    }
}

/// 角色基础组件（预留）
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct Character {
    /// 角色姓名
    name: String,
    /// 角色年龄
    age: u8,
}

/// 功法组件（预留）
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct Cultivation {
    /// 当前境界
    pub realm: Realm,
    /// 当前真气
    pub qi: f32,
    /// 真气上限
    pub max_qi: f32,
}

/// 修仙境界枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Realm {
    /// 凡人
    Mortal,
    /// 练气
    QiRefining,
    /// 筑基
    Foundation,
    /// 金丹
    GoldenCore,
    /// 元婴
    NascentSoul,
}

impl Default for Cultivation {
    fn default() -> Self {
        Self {
            realm: Realm::Mortal,
            qi: 0.0,
            max_qi: 100.0,
        }
    }
}
