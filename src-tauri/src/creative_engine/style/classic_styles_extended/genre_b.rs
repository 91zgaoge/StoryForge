use super::super::dna::*;

/// 黑暗奇幻风格
/// 特征：残酷、灰色道德、血腥、现实主义魔法
pub fn grimdark() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "黑暗奇幻".to_string(),
            author: None,
            description: "残酷现实主义奇幻，灰色道德，血腥暴力，魔法代价高昂，世界黑暗".to_string(),
            genre_association: Some("奇幻/黑暗".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "战争术语".to_string(),
                "酷刑词汇".to_string(),
                "政治术语".to_string(),
                "粗俗语".to_string(),
            ],
            signature_words: vec![
                "血".to_string(),
                "背叛".to_string(),
                "权力".to_string(),
                "死亡".to_string(),
            ],
            avoided_patterns: vec!["童话氛围".to_string(), "英雄光环".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 26,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "冷酷直接，节奏紧凑".to_string(),
            preferred_structures: vec![
                "多视角".to_string(),
                "政治博弈".to_string(),
                "战斗描写".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "直接冷酷，短句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.04,
            preferred_devices: vec!["讽刺".to_string(), "对比".to_string()],
            imagery_preference: vec![
                "战争意象".to_string(),
                "政治意象".to_string(),
                "黑暗意象".to_string(),
            ],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.25,
            omniscience_level: 0.5,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "冷酷绝望".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "威胁".to_string(),
                "政治谈判".to_string(),
                "粗口".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 仙侠修真风格
/// 特征：古风、升级、世界观、丹药法宝
pub fn xianxia() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "仙侠修真".to_string(),
            author: None,
            description: "东方玄幻修真体系，境界升级，丹药法宝，宗门派系，古风语言".to_string(),
            genre_association: Some("仙侠/玄幻".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "修真术语".to_string(),
                "丹药名称".to_string(),
                "法宝词汇".to_string(),
                "境界称谓".to_string(),
            ],
            signature_words: vec![
                "境界".to_string(),
                "灵气".to_string(),
                "法宝".to_string(),
                "突破".to_string(),
            ],
            avoided_patterns: vec!["现代科技".to_string(), "西方术语".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 28,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "古风流畅，战斗紧凑".to_string(),
            preferred_structures: vec![
                "境界说明".to_string(),
                "战斗描写".to_string(),
                "功法描述".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "传统标点，战斗短句".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["比喻".to_string(), "夸张".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "神话意象".to_string(),
                "战斗意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.3,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "热血执念".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "古风对白".to_string(),
                "宗门规矩".to_string(),
                "挑衅".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 无限流风格
/// 特征：副本、惊悚、智斗、系统提示
pub fn infinite_flow() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "无限流".to_string(),
            author: None,
            description: "副本闯关体系，惊悚生存，智斗博弈，系统提示，数据面板，团队协作"
                .to_string(),
            genre_association: Some("无限流/惊悚".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "游戏术语".to_string(),
                "恐怖元素".to_string(),
                "系统提示".to_string(),
                "数据词汇".to_string(),
            ],
            signature_words: vec![
                "副本".to_string(),
                "系统".to_string(),
                "积分".to_string(),
                "生存".to_string(),
            ],
            avoided_patterns: vec!["抒情议论".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 22,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "紧张刺激，信息轰炸".to_string(),
            preferred_structures: vec![
                "系统提示".to_string(),
                "规则说明".to_string(),
                "智斗推演".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "快节奏，括号注释".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.02,
            preferred_devices: vec!["悬念".to_string(), "伏笔".to_string()],
            imagery_preference: vec!["恐怖意象".to_string(), "游戏意象".to_string()],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.25,
            omniscience_level: 0.2,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "紧张刺激".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "dry".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.35,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "系统提示音".to_string(),
                "团队战术".to_string(),
                "规则讨论".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}
