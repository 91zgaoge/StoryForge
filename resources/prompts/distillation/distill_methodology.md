---
id: distill_methodology
name: "创作方法论结构化生成"
description: "把合并后的创作原则组织成带步骤的创作方法论"
category: distillation
version: 0.33.2
variables:
  - principles
  - book_title
---

你是一位小说创作方法论专家。以下是从指导书《{{book_title}}》提炼的核心创作原则，请把它们组织成一套**分步骤执行**的创作方法论，供 AI 在续写小说时逐步应用。

要求：
1. name: 方法论名称，不超过 12 个字，体现该书核心思想（如"三幕冲突驱动法"）
2. description: 一句话描述该方法论的适用场景与核心价值
3. steps: 3-8 个执行步骤，按创作先后顺序排列；每个步骤包含：
   - title: 步骤名称（不超过 10 个字）
   - instruction: 该步骤的详细执行指引（100-200 字，直接以第二人称指令语气写给执行者，包含该步骤要运用的原则）
   - checklist: 2-4 条该步骤完成质量的自检项（每条一句话，疑问句或判断句）
4. 所有原则必须被分配到某个步骤中，不得遗漏核心思想
5. 只输出 JSON，不要有任何其他文字

核心创作原则：
{{principles}}

JSON格式：
{"name":"...","description":"...","steps":[{"title":"...","instruction":"...","checklist":["..."]}]}
