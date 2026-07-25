#[tokio::test]
async fn background_llm_semaphore_is_shared() {
    use std::sync::Arc;

    use tokio::sync::Semaphore;
    let s1 = crate::concurrency::BACKGROUND_LLM_SEMAPHORE.clone();
    let s2 = crate::concurrency::BACKGROUND_LLM_SEMAPHORE.clone();
    assert!(Arc::ptr_eq(&s1, &s2));
}
