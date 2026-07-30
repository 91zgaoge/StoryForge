---
id: novel_creation_world_options
name: "创世向导-世界观选项"
description: "NovelCreationAgent：生成多个世界观概念供用户选择"
category: creation
version: 0.30.42
variables:
---

作为一位资深世界观设计师，请基于以下用户输入，创建{{count}}个独特的世界观概念。

用户输入：{{input}}

请用 JSON 格式回复，格式如下：
{
  "world_buildings": [
    {
      "id": "wb_1",
      "concept": "世界观核心概念（20-50字）",
      "rules": [
        {"name": "规则名称", "description": "规则描述", "rule_type": "Magic", "importance": 8}
      ],
      "history": "历史背景（100-200字）",
      "cultures": [
        {"name": "文化名称", "description": "文化描述", "customs": ["习俗1", "习俗2"], "values": ["价值观1", "价值观2"]}
      ]
    }
  ]
}

要求：
1. 每个世界观应该有独特的核心概念
2. 包含基本的世界规则（3-5条）
3. 有历史背景概述
4. 包含2-3个主要文化设定
5. 世界观类型可以是：玄幻、科幻、都市、历史、武侠、悬疑等
6. 规则类型包括：Magic（魔法）、Technology（科技）、Social（社会）、Physical（物理）
7. importance 范围 1-10

格式约束（必须严格遵守，否则解析会失败）：
- 只输出纯 JSON，禁止使用 markdown 代码块包裹（不要输出 ```json 或 ``` 标记）
- 字符串值内部若需引用文字，使用中文引号「」或转义双引号 \"，禁止使用未转义的裸双引号
- 不要在 JSON 之外输出任何解释、前言、后记
- 确保 JSON 格式正确，可直接被解析
