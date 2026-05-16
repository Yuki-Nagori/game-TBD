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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_time_default() {
        let time = GameTime::default();
        assert_eq!(time.year, 1573, "默认年份应为万历元年");
        assert_eq!(time.month, 1, "默认月份应为1月");
        assert_eq!(time.day, 1, "默认日期应为1日");
        assert_eq!(time.hour, 6.0, "默认小时应为6时");
    }

    #[test]
    fn test_game_time_advance_within_day() {
        let mut time = GameTime::default();
        time.advance(2.0);
        assert_eq!(time.hour, 8.0, "推进2小时后应为8时");
        assert_eq!(time.day, 1, "未跨天日期不变");
    }

    #[test]
    fn test_game_time_advance_crosses_day_boundary() {
        let mut time = GameTime {
            year: 1573,
            month: 1,
            day: 1,
            hour: 23.0,
        };
        time.advance(2.0);
        assert_eq!(time.hour, 1.0, "跨天后小时应从0开始累加");
        assert_eq!(time.day, 2, "跨天后日期应递增");
    }

    #[test]
    fn test_game_time_advance_crosses_month_boundary() {
        let mut time = GameTime {
            year: 1573,
            month: 1,
            day: 30,
            hour: 23.0,
        };
        time.advance(2.0);
        assert_eq!(time.month, 2, "跨月后月份应递增");
        assert_eq!(time.day, 1, "跨月后日期应从1开始");
        assert_eq!(time.hour, 1.0, "跨月后小时应正确");
    }

    #[test]
    fn test_game_time_advance_crosses_year_boundary() {
        let mut time = GameTime {
            year: 1573,
            month: 12,
            day: 30,
            hour: 23.0,
        };
        time.advance(2.0);
        assert_eq!(time.year, 1574, "跨年后年份应递增");
        assert_eq!(time.month, 1, "跨年后月份应从1开始");
        assert_eq!(time.day, 1, "跨年后日期应从1开始");
    }

    #[test]
    fn test_game_time_advance_multiple_months() {
        let mut time = GameTime {
            year: 1573,
            month: 1,
            day: 1,
            hour: 0.0,
        };
        // 推进 61 天 = 2个月1天（每月30天）
        time.advance(24.0 * 61.0);
        assert_eq!(time.year, 1573, "未跨年");
        assert_eq!(time.month, 3, "应推进2个月");
        assert_eq!(time.day, 2, "应剩余1天");
    }

    #[test]
    fn test_game_time_advance_negative_hours() {
        let mut time = GameTime {
            year: 1573,
            month: 1,
            day: 2,
            hour: 6.0,
        };
        time.advance(-2.0);
        assert_eq!(time.hour, 4.0, "负值推进应回退小时");
        assert_eq!(time.day, 2, "未跨天日期不变");
    }

    #[test]
    fn test_game_time_display_format() {
        let time = GameTime {
            year: 1573,
            month: 6,
            day: 15,
            hour: 14.5,
        };
        let display = format!("{}", time);
        assert_eq!(
            display, "1573年06月15日 14:00",
            "Display 应截断小时为整数并补零"
        );
    }

    #[test]
    fn test_game_time_display_year_padding() {
        let time = GameTime {
            year: 999,
            month: 1,
            day: 1,
            hour: 0.0,
        };
        let display = format!("{}", time);
        assert_eq!(display, "0999年01月01日 00:00", "年份应补零至4位");
    }

    #[test]
    fn test_cultivation_default() {
        let cult = Cultivation::default();
        assert_eq!(cult.realm, Realm::Mortal, "默认境界应为凡人");
        assert_eq!(cult.qi, 0.0, "默认真气应为0");
        assert_eq!(cult.max_qi, 100.0, "默认真气上限应为100");
    }

    #[test]
    fn test_realm_variants_exist() {
        // 验证所有境界变体可构造且互不相等
        let realms = [
            Realm::Mortal,
            Realm::QiRefining,
            Realm::Foundation,
            Realm::GoldenCore,
            Realm::NascentSoul,
        ];
        for (i, r1) in realms.iter().enumerate() {
            for (j, r2) in realms.iter().enumerate() {
                if i == j {
                    assert_eq!(r1, r2, "同一境界应相等");
                } else {
                    assert_ne!(r1, r2, "不同境界应不相等");
                }
            }
        }
    }

    #[test]
    fn test_character_default_construction() {
        // Character 无 Default，但可通过结构体字面量构造
        let character = Character {
            name: "测试角色".to_string(),
            age: 20,
        };
        assert_eq!(character.name, "测试角色");
        assert_eq!(character.age, 20);
    }
}
