//! 核心模块集成测试

use ming_rpg::core::{Cultivation, GameTime, Realm};

#[test]
fn test_game_time_advance() {
    let mut time = GameTime::default();
    time.advance(48.0); // 推进2天

    assert_eq!(time.day, 3);
    assert_eq!(time.hour, 6.0);
}

#[test]
fn test_cultivation_default() {
    let cult = Cultivation::default();
    assert_eq!(cult.realm, Realm::Mortal);
    assert_eq!(cult.qi, 0.0);
}
