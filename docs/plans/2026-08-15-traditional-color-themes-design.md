# 传统色主题（纸·帘·印）设计

日期：2026-08-15  
范围：幕前墨纸 + 幕后机械配色。不搬 dsh-theme-plugin 的 89 令牌或生成器。

## 产品

- 12 套写作向传统色（dsh 精选去掉玫红 / 紫云 / 浅紫藤萝，补鷃蓝 / 皮弁 / 汉绣绿）。
- 用色纪律：纸 = 写作面材料；帘 = 侧栏/次级面认色；印 = 焦点即锚色本人（`--gold` / `--cinema-gold` 不再偏相 40°）。印色（seal）只进 `--cinema-velvet`。
- 幕前 / 幕后分选：色点只改幕前；设置页两列，点左不影响右。
- 旧四套迁移：warm→zhuhong，cool→qunqing，amber→tenghuang，indigo→daizi。

## 身份

`zhuqing | zhuhong | qunqing | tenghuang | jiangzi | lingmenghong | heyelv | fenlv | daizi | yanlan | pibian | hanxiulv`

每个 id：亮纸只给幕前，暗机械只给幕后。默认 `zhuhong`。

## 存储与事件

- `storymoss-color-theme-front` / `storymoss-color-theme-back`
- 旧 key `storymoss-color-theme` 仅在新 key 皆空时读一次，写入两边
- 事件 `{ surface: "front" | "back", id }`

## 色值来源

dsh-theme-plugin MIT，抄 12×2 已通过闸门的 paper / sidebar / brand / ink / layers / seal。词表仍是草苔 `--parchment*` / `--terracotta*` / `--cinema-*`。

## 不变量

1. 所有主题 `gold === terracotta`（幕前）且 `cinema-gold` 为该色暗面 brand。
2. 改幕前不写 back key、不 `applyBackstageTheme`；改幕后不对偶。
3. `--ai-accent-tint` 跟随当前窗强调色，禁止写死暖赭。
