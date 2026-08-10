---
id: novel_creation_character_roster
name: "创世向导-角色谱生成"
description: "NovelCreationAgent：生成多组角色配置供用户选择"
category: creation
version: 0.26.28
variables:
---

作为一位角色设计专家，请基于以下世界观，创建{{count}}组不同的角色配置。

世界观：{{input}}

要求：
1. 每组包含3-5个核心角色
2. 角色应该代表不同的立场和功能（主角、反派、导师、盟友等）
3. 角色性格应该鲜明，有冲突和互补
4. 考虑世界观对角色塑造的影响

请以JSON格式输出，格式如下：
{
  "character_sets": [
    [
      {
        "id": "char_1_1",
        "name": "角色姓名",
        "personality": "性格特点（30-50字）",
        "background": "背景故事（50-100字）",
        "goals": "目标动机（30-50字）",
        "voice_style": "语言风格（20-30字）",
        "emotional_core": "情感内核：主导情感倾向（20-40字）",
        "emotional_trigger": "情感触发：引爆情绪的场景或行为（20-40字）",
        "emotional_wound": "情感创伤：塑造情感模式的过往伤口（20-40字）",
        "emotional_need": "情感需求：深层渴望的情感满足（20-40字）"
      }
    ]
  ]
}

注意：
- 每个角色必须含全部10个字段，emotional_* 四字段不得为空
- 姓名应符合世界观文化背景，且具有辨识度
- 禁止使用林、陈、王、李、张、刘等最常见单字姓；禁止单字名
- 同一组角色姓氏不得重复
- 角色应有明确外貌、性别、年龄描述，避免千人一面
- 每组角色之间应有内在联系和冲突
- 确保JSON格式正确，只输出 JSON
