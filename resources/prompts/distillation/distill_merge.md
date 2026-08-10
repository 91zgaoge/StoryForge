---
id: distill_merge
name: "指导书资产分类合并"
description: "合并全书提炼的四类创作资产，分类去重并保留最具操作性的条目"
category: distillation
version: 0.36.0
variables:
  - points
---

你是一位小说创作方法论专家。以下是从一本故事创作指导书各章节提炼出的创作资产（分【要点】【技巧】【决策规则】【反模式】四类），请分类合并去重。

要求：
1. 语义相同的条目合并为一条，保留最准确的表述与作者原始命名
2. principles：按主题归类排序（冲突设计、人物塑造、结构节奏、世界观、对白等），保留最重要的 10-20 条，每条一句话
3. techniques：保留最实用、最具操作性的 5-15 条，每条必须含 name/when_to_use/how 三个字段
4. decision_rules：保留 5-10 条，保持"当X时做Y，因为Z"格式
5. anti_patterns：保留 3-8 条，每条含 what/why 两个字段
6. 只输出 JSON，不要有任何其他文字

原始资产列表：
{{points}}

JSON格式：
{"principles":["原则1","原则2"],"techniques":[{"name":"技巧名","when_to_use":"何时使用","how":"具体怎么做"}],"decision_rules":["当X时做Y，因为Z"],"anti_patterns":[{"what":"应避免的做法","why":"为什么会导致失败"}]}
