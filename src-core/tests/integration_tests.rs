#[cfg(test)]
mod tests {
    use storyforge_core::ComplexityTier;

    #[test]
    fn test_complexity_classification() {
        let simple = "Write a short scene";
        assert_eq!(ComplexityTier::classify_from_prompt(simple), ComplexityTier::Low);

        let complex = "Analyze the climax and revelation";
        assert_eq!(ComplexityTier::classify_from_prompt(complex), ComplexityTier::Critical);
    }
}
