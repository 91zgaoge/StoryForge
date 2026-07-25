use super::super::dna::*;

/// 白先勇风格
/// 特征：台北人、苍凉精致、旧贵族、细腻心理
pub fn bai_xianyong() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "白先勇".to_string(),
            author: Some("白先勇".to_string()),
            description: "台湾作家，以精致细腻的笔触写流亡贵族的没落，苍凉华美，心理深度"
                .to_string(),
            genre_association: Some("现代小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "色彩词汇".to_string(),
                "服饰细节".to_string(),
                "戏曲术语".to_string(),
                "贵族用语".to_string(),
            ],
            signature_words: vec![
                "游园".to_string(),
                "惊梦".to_string(),
                "繁华".to_string(),
                "没落".to_string(),
            ],
            avoided_patterns: vec!["粗俗口语".to_string(), "直白议论".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 30,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "舒缓精致，如昆曲水磨调".to_string(),
            preferred_structures: vec![
                "长句铺陈".to_string(),
                "倒叙".to_string(),
                "意象叠加".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "精致，善用逗号与分号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.1,
            preferred_devices: vec!["象征".to_string(), "比喻".to_string(), "通感".to_string()],
            imagery_preference: vec![
                "戏曲意象".to_string(),
                "色彩意象".to_string(),
                "繁华意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.3,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.06,
            dominant_mood: "苍凉精致".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "古典白话".to_string(),
                "戏曲化语言".to_string(),
                "欲言又止".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 钱锺书风格
/// 特征：博学讽刺、机智俏皮、比喻密集、学贯中西
pub fn qian_zhongshu() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "钱锺书".to_string(),
            author: Some("钱锺书".to_string()),
            description: "学贯中西的讽刺大师，以博学和机智写知识分子的困境，比喻奇警，旁征博引"
                .to_string(),
            genre_association: Some("讽刺小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "学术术语".to_string(),
                "西方典故".to_string(),
                "古典诗词".to_string(),
                "讽刺语汇".to_string(),
            ],
            signature_words: vec![
                "围城".to_string(),
                "文凭".to_string(),
                "留学".to_string(),
                "教授".to_string(),
            ],
            avoided_patterns: vec!["乡土土语".to_string(), "直白抒情".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 38,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "洋洋洒洒，妙趣横生".to_string(),
            preferred_structures: vec![
                "长句嵌套".to_string(),
                "对比反讽".to_string(),
                "引经据典".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "繁复精致，善用逗号分号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.14,
            preferred_devices: vec![
                "比喻".to_string(),
                "反讽".to_string(),
                "用典".to_string(),
                "夸张".to_string(),
            ],
            imagery_preference: vec![
                "学术意象".to_string(),
                "西方意象".to_string(),
                "婚姻意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.9,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "机智讽刺".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.35,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.7,
            signature_patterns: vec![
                "机锋往来".to_string(),
                "中西夹杂".to_string(),
                "知识分子腔".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 郁达夫风格
/// 特征：抒情自叙、感伤独白、浪漫颓废、心理暴露
pub fn yu_dafu() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "郁达夫".to_string(),
            author: Some("郁达夫".to_string()),
            description: "创造社代表，以自叙传体式写青年苦闷，感伤抒情，心理暴露，浪漫颓废"
                .to_string(),
            genre_association: Some("抒情小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "情感词汇".to_string(),
                "自然意象".to_string(),
                "病态词汇".to_string(),
                "西洋语汇".to_string(),
            ],
            signature_words: vec![
                "沉沦".to_string(),
                "孤独".to_string(),
                "眼泪".to_string(),
                "秋".to_string(),
            ],
            avoided_patterns: vec!["客观叙事".to_string(), "社会分析".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 34,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "抒情长句，叹词频繁".to_string(),
            preferred_structures: vec![
                "独白".to_string(),
                "感叹".to_string(),
                "排比抒情".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "感叹号、省略号、破折号密集".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.1,
            preferred_devices: vec!["拟人".to_string(), "象征".to_string(), "呼告".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "病态意象".to_string(),
                "孤独意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.6,
            omniscience_level: 0.0,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.1,
            dominant_mood: "感伤颓废".to_string(),
            emotional_arc_pattern: "cyclical".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.15,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "自言自语".to_string(),
                "欲说还休".to_string(),
                "书信体".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}
