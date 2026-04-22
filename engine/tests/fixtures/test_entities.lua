-- 测试实体夹具
-- 用于集成测试中的实体创建和位置设置

local e1 = Entity.create("npc")
local e2 = Entity.create("npc")
local e3 = Entity.create("player")

Entity.set_position(e1.id, 1.0, 2.0, 3.0)
Entity.set_position(e2.id, 4.0, 5.0, 6.0)
Entity.set_position(e3.id, 7.0, 8.0, 9.0)
