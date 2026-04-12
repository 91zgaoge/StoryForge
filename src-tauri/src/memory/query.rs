//! 四阶段查询检索管线
//! 
//! Stage 1: CJK二元组分词搜索
//! Stage 2: 图谱扩展
//! Stage 3: 预算控制
//! Stage 4: 带引用编号的上下文组装

use super::tokenizer::CJKTokenizer;
use crate::db::models_v3::{Entity, Relation};
use std::sync::Arc;

/// 查询管道
pub struct QueryPipeline {
    tokenizer: CJKTokenizer,
    budget_config: BudgetConfig,
}

/// 预算配置
#[derive(Debug, Clone)]
pub struct BudgetConfig {
    /// 总token预算 (4K-1M可配)
    pub total_budget: usize,
    /// 搜索预算比例 (60%)
    pub search_budget_pct: f32,
    /// 图谱预算比例 (20%)
    pub graph_budget_pct: f32,
    /// 上下文预算比例 (5%)
    pub context_budget_pct: f32,
    /// 组装预算比例 (15%)
    pub assembly_budget_pct: f32,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            total_budget: 4096,
            search_budget_pct: 0.60,
            graph_budget_pct: 0.20,
            context_budget_pct: 0.05,
            assembly_budget_pct: 0.15,
        }
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub source_type: SourceType,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Scene,
    Entity,
    Memory,
}

/// 图谱扩展结果
#[derive(Debug, Clone)]
pub struct GraphResult {
    pub entity: Entity,
    pub relation_strength: f32,
    pub related_entities: Vec<(Entity, f32)>,
}

/// 选中的上下文
#[derive(Debug, Clone)]
pub struct SelectedContext {
    pub content: String,
    pub source: String,
    pub citation_number: usize,
    pub relevance_score: f32,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub context: String,
    pub citations: Vec<Citation>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct Citation {
    pub number: usize,
    pub source: String,
    pub preview: String,
}

impl QueryPipeline {
    pub fn new(budget_config: BudgetConfig) -> Self {
        Self {
            tokenizer: CJKTokenizer::new(),
            budget_config,
        }
    }

    /// 四阶段查询检索
    pub async fn query(
        &self,
        query: &str,
        story_id: &str,
        vector_store: &dyn VectorStore,
        knowledge_graph: &dyn KnowledgeGraph,
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        // Stage 1: CJK二元组分词搜索
        let search_results = self.token_search(query, story_id, vector_store).await?;
        
        // Stage 2: 图谱扩展
        let graph_expansion = self.graph_expansion(&search_results, knowledge_graph).await?;
        
        // Stage 3: 预算控制
        let selected = self.budget_control(&search_results, &graph_expansion)?;
        
        // Stage 4: 带引用编号的上下文组装
        let result = self.assemble_context(&selected)?;
        
        Ok(result)
    }

    /// Stage 1: CJK二元组分词搜索
    async fn token_search(
        &self,
        query: &str,
        story_id: &str,
        vector_store: &dyn VectorStore,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        // 对查询进行CJK二元组分词
        let tokens = self.tokenizer.tokenize(query);
        
        // 在向量存储中进行多token搜索
        let mut all_results = vec![];
        
        for token in tokens {
            let results = vector_store.search_with_token(story_id, &token, 10).await?;
            all_results.extend(results);
        }
        
        // 去重并按分数排序
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_results.dedup_by(|a, b| a.id == b.id);
        
        // 返回Top 50
        Ok(all_results.into_iter().take(50).collect())
    }

    /// Stage 2: 图谱扩展
    async fn graph_expansion(
        &self,
        search_results: &[SearchResult],
        knowledge_graph: &dyn KnowledgeGraph,
    ) -> Result<Vec<GraphResult>, Box<dyn std::error::Error>> {
        let mut expanded = vec![];
        let mut processed_entities = std::collections::HashSet::new();
        
        for result in search_results {
            // 从搜索结果中提取实体
            if let Ok(entity) = knowledge_graph.find_entity_by_name(&result.content).await {
                if processed_entities.insert(entity.id.clone()) {
                    // 获取相关实体（基于关系强度）
                    let related = knowledge_graph
                        .get_related_entities(&entity.id, 0.3)
                        .await?;
                    
                    // 计算加权分数
                    let related_with_scores: Vec<(Entity, f32)> = related
                        .into_iter()
                        .map(|(e, strength)| {
                            let weighted_score = strength * 0.8 + result.score * 0.2;
                            (e, weighted_score)
                        })
                        .collect();
                    
                    expanded.push(GraphResult {
                        entity,
                        relation_strength: result.score,
                        related_entities: related_with_scores,
                    });
                }
            }
        }
        
        // 按关系强度排序
        expanded.sort_by(|a, b| {
            let a_score = a.relation_strength + 
                a.related_entities.iter().map(|(_, s)| s).sum::<f32>();
            let b_score = b.relation_strength + 
                b.related_entities.iter().map(|(_, s)| s).sum::<f32>();
            b_score.partial_cmp(&a_score).unwrap()
        });
        
        Ok(expanded)
    }

    /// Stage 3: 预算控制
    fn budget_control(
        &self,
        search_results: &[SearchResult],
        graph_expansion: &[GraphResult],
    ) -> Result<Vec<SelectedContext>, Box<dyn std::error::Error>> {
        let total_budget = self.budget_config.total_budget;
        let search_budget = (total_budget as f32 * self.budget_config.search_budget_pct) as usize;
        let graph_budget = (total_budget as f32 * self.budget_config.graph_budget_pct) as usize;
        
        let mut selected = vec![];
        let mut used_budget = 0;
        let mut citation_counter = 1;
        
        // 优先选择搜索结果的Top-K
        for result in search_results.iter().take(10) {
            let cost = result.content.len();
            if used_budget + cost > search_budget {
                break;
            }
            
            selected.push(SelectedContext {
                content: result.content.clone(),
                source: format!("{:?}", result.source_type),
                citation_number: citation_counter,
                relevance_score: result.score,
            });
            
            used_budget += cost;
            citation_counter += 1;
        }
        
        // 然后选择图谱扩展结果
        for graph_result in graph_expansion {
            // 添加主实体
            let entity_desc = format!("{}: {}", 
                graph_result.entity.name,
                graph_result.entity.attributes.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("无描述")
            );
            let cost = entity_desc.len();
            
            if used_budget + cost <= search_budget + graph_budget {
                selected.push(SelectedContext {
                    content: entity_desc,
                    source: format!("Entity: {}", graph_result.entity.name),
                    citation_number: citation_counter,
                    relevance_score: graph_result.relation_strength,
                });
                used_budget += cost;
                citation_counter += 1;
            }
            
            // 添加相关实体（预算允许的情况下）
            for (related, score) in &graph_result.related_entities {
                let related_desc = format!("{}: {}",
                    related.name,
                    related.attributes.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("无描述")
                );
                let cost = related_desc.len();
                
                if used_budget + cost <= search_budget + graph_budget {
                    selected.push(SelectedContext {
                        content: related_desc,
                        source: format!("Related: {}", related.name),
                        citation_number: citation_counter,
                        relevance_score: *score,
                    });
                    used_budget += cost;
                    citation_counter += 1;
                } else {
                    break;
                }
            }
        }
        
        Ok(selected)
    }

    /// Stage 4: 带引用编号的上下文组装
    fn assemble_context(
        &self,
        selected: &[SelectedContext],
    ) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let mut context_parts = vec![];
        let mut citations = vec![];
        let mut total_tokens = 0;
        
        for item in selected {
            let part = format!("[{}] {}\n", item.citation_number, item.content);
            total_tokens += part.len();
            
            context_parts.push(part);
            
            citations.push(Citation {
                number: item.citation_number,
                source: item.source.clone(),
                preview: item.content.chars().take(50).collect::<String>() + "...",
            });
        }
        
        Ok(QueryResult {
            context: context_parts.join("\n"),
            citations,
            total_tokens,
        })
    }
}

/// 向量存储接口（用于查询）
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    async fn search_with_token(
        &self,
        story_id: &str,
        token: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
}

/// 知识图谱接口（用于查询）
#[async_trait::async_trait]
pub trait KnowledgeGraph: Send + Sync {
    async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> Result<Entity, Box<dyn std::error::Error>>;
    
    async fn get_related_entities(
        &self,
        entity_id: &str,
        min_strength: f32,
    ) -> Result<Vec<(Entity, f32)>, Box<dyn std::error::Error>>;
}
