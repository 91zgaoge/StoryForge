---
id: distill_metadata
name: "指导书元信息识别"
description: "识别故事创作指导书的标题、作者与主题"
category: distillation
version: 0.33.2
variables:
  - text
---

请分析以下书籍开头，识别这是一本什么书。只输出 JSON，不要有任何其他文字。

要求：
1. title: 书名（如无法确定则为null）
2. author: 作者名（如无法确定则为null）
3. subject: 本书主题的一句话概括（例如"小说冲突设计""人物塑造方法"）

文本样本：
{{text}}

JSON格式：
{"title":"...","author":"...","subject":"..."}
