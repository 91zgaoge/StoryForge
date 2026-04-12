# 已加载的技能

## 本机技能 (~/.claude/skills/)

### 1. json-canvas
- **用途**: 创建和编辑 Obsidian Canvas 文件 (.canvas)
- **能力**: 
  - 创建节点（text/file/link/group）
  - 添加边/连接
  - 设置颜色、位置、大小
  - 生成思维导图、流程图、项目板
- **文件**: `C:/Users/admin/.claude/skills/json-canvas/SKILL.md`

### 2. office
- **用途**: 生成 Office 文档 (DOCX/XLSX/PDF/PPTX)
- **能力**:
  - Word 文档生成 (docx)
  - Excel 表格生成 (xlsx)
  - PDF 文档生成 (pdf-lib)
  - PowerPoint 演示文稿 (pptxgenjs)
  - GB/T 9704-2012 中国公文格式
- **文件**: `C:/Users/admin/.claude/skills/office/SKILL.md`

### 3. obsidian-markdown
- **用途**: Obsidian 风格 Markdown 编辑
- **能力**:
  - Wikilinks 内部链接 `[[Note]]`
  - Callouts 标注块 `[!note]`
  - Embeds 嵌入 `![[file]]`
  - Properties/Frontmatter
  - Mermaid 图表
  - LaTeX 数学公式
- **文件**: `C:/Users/admin/.claude/skills/obsidian-markdown/SKILL.md`

### 4. brainstorming (已安装)
- **用途**: 需求分析和头脑风暴
- **位置**: `~/.claude/skills/brainstorming/`

### 5. find-skills (已安装)
- **用途**: 技能发现和推荐
- **位置**: `~/.claude/skills/find-skills/`

### 6. frontend-design (已安装)
- **用途**: 前端设计系统指导
- **位置**: `~/.claude/skills/frontend-design/`

### 7. skill-creator (已安装)
- **用途**: 创建自定义技能
- **位置**: `~/.claude/skills/skill-creator/`

### 8. systematic-debugging (已安装)
- **用途**: 系统化的调试方法
- **位置**: `~/.claude/skills/systematic-debugging/`

### 9. vercel-react-native-skills (已安装)
- **用途**: React Native 最佳实践
- **位置**: `~/.claude/skills/vercel-react-native-skills/`

## 全局技能 (~/.agents/skills/)

- brainstorming
- find-skills
- office
- react-components
- skill-creator
- systematic-debugging
- vercel-react-native-skills

## 使用建议

### 对于 StoryForge 项目

| 场景 | 推荐技能 |
|------|---------|
| 创建故事大纲可视化 | json-canvas |
| 导出故事为文档 | office |
| 编写技术文档 | obsidian-markdown |
| 新功能设计 | brainstorming |
| Bug 修复 | systematic-debugging |
| UI 改进 | frontend-design |

## 技能加载记录

- **首次加载**: 2026-04-12
- **新增技能**: json-canvas, office, obsidian-markdown
- **加载者**: 用户授权
