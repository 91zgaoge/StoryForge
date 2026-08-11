---
id: distill_foldin
name: "指导书资产增量融合"
description: "将新指导书提炼的资产与现有方法论资产融合去重，产出更强的统一资产集"
category: distillation
version: 0.36.0
variables:
  - existing
  - new
---

你是一位小说创作方法论专家。一位创作者已有一套从指导书提炼的创作方法论资产，现在又提炼了一本新指导书的资产。请将两者**融合为一套更强的统一资产**。

要求：
1. 语义相同的条目合并为一条，保留最准确的表述与作者原始命名（两本书对同一技巧命名不同时，并列保留如"场景目标（SCU）"）
2. 冲突的指导（一本说应该X、另一本说应该非X）两条都保留，并在表述中注明适用情境差异
3. principles：按主题归类排序，保留最重要的 15-25 条
4. techniques：保留最实用、最具操作性的 8-20 条，每条必须含 name/when_to_use/how
5. decision_rules：保留 6-12 条，保持"当X时做Y，因为Z"格式
6. anti_patterns：保留 4-10 条，每条含 what/why
7. 只输出 JSON，不要有任何其他文字

【现有方法论资产】
{{existing}}

【新提炼资产】
{{new}}

JSON格式：
{"principles":["原则1","原则2"],"techniques":[{"name":"技巧名","when_to_use":"何时使用","how":"具体怎么做"}],"decision_rules":["当X时做Y，因为Z"],"anti_patterns":[{"what":"应避免的做法","why":"为什么会导致失败"}]}
