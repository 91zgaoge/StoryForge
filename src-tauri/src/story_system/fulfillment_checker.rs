//! 章节合同履行度检查
//!
//! 纯同步、启发式：检查正文是否覆盖章节合同的 must_cover_nodes、
//! 是否触及 forbidden_zones，并给出 0.0-1.0 的履行度评分。
//!
//! 注意：具体评分逻辑已下沉到 `crate::domain::contracts::RuntimeContract`，
//! 本模块保留函数入口以兼容现有调用方。

pub use crate::domain::contracts::FulfillmentResult;
use crate::domain::contracts::RuntimeContract;

/// 评估正文对运行时合同的履行情况。
///
/// 委托给 `RuntimeContract::evaluate_fulfillment` 以保持业务规则在 domain 层。
pub fn evaluate_contract_fulfillment(
    content: &str,
    contract: &RuntimeContract,
) -> FulfillmentResult {
    contract.evaluate_fulfillment(content)
}
