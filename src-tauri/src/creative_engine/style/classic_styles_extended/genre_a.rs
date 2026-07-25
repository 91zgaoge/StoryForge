use super::super::dna::*;

/// 赛博朋克风格
/// 特征：高科技低生活、霓虹、碎片化、黑客
pub fn cyberpunk() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "赛博朋克".to_string(),
            author: None,
            description: "高科技与低生活的黑暗融合，霓虹灯雨夜，信息过载，身体改造，企业霸权"
                .to_string(),
            genre_association: Some("科幻/赛博朋克".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "futuristic".to_string(),
            preferred_categories: vec![
                "科技术语".to_string(),
                "日语借词".to_string(),
                "毒品词汇".to_string(),
                "网络用语".to_string(),
            ],
            signature_words: vec![
                "赛博空间".to_string(),
                "神经接口".to_string(),
                "霓虹".to_string(),
                "黑客".to_string(),
            ],
            avoided_patterns: vec!["田园牧歌".to_string(), "温情脉脉".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 20,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "快节奏，信息密集".to_string(),
            preferred_structures: vec![
                "碎片化叙事".to_string(),
                "技术说明".to_string(),
                "视角跳跃".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "极简，短句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.04,
            preferred_devices: vec!["比喻".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "科技意象".to_string(),
                "城市意象".to_string(),
                "身体意象".to_string(),
            ],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.0,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.02,
            dominant_mood: "冷漠疏离".to_string(),
            emotional_arc_pattern: "static".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "黑客黑话".to_string(),
                "街头 slang".to_string(),
                "日语借词".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 蒸汽朋克风格
/// 特征：维多利亚、齿轮、冒险、绅士风度
pub fn steampunk() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "蒸汽朋克".to_string(),
            author: None,
            description: "维多利亚时代与蒸汽科技的浪漫融合，齿轮飞艇，绅士冒险，复古未来"
                .to_string(),
            genre_association: Some("科幻/蒸汽朋克".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "机械术语".to_string(),
                "维多利亚用语".to_string(),
                "航海词汇".to_string(),
                "绅士用语".to_string(),
            ],
            signature_words: vec![
                "蒸汽".to_string(),
                "齿轮".to_string(),
                "飞艇".to_string(),
                "绅士".to_string(),
            ],
            avoided_patterns: vec!["现代俚语".to_string(), "电子词汇".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 30,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "优雅冒险，如老式探险小说".to_string(),
            preferred_structures: vec![
                "场景描写".to_string(),
                "技术说明".to_string(),
                "绅士对话".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "传统精致，长句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["比喻".to_string(), "夸张".to_string()],
            imagery_preference: vec![
                "机械意象".to_string(),
                "维多利亚意象".to_string(),
                "冒险意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.1,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "浪漫冒险".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "维多利亚腔".to_string(),
                "绅士风度".to_string(),
                "冒险术语".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 新怪谈风格
/// 特征：都市奇幻、不可解、日常恐怖、官僚迷宫
pub fn new_weird() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "新怪谈".to_string(),
            author: None,
            description: "当代恐怖美学，以不可解的异常渗透日常，官僚机构，档案体，理性崩塌"
                .to_string(),
            genre_association: Some("恐怖/都市奇幻".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "官僚术语".to_string(),
                "建筑词汇".to_string(),
                "档案用语".to_string(),
                "异常描述".to_string(),
            ],
            signature_words: vec![
                "异常".to_string(),
                "档案".to_string(),
                "阈限".to_string(),
                "不可解".to_string(),
            ],
            avoided_patterns: vec!["传统鬼怪".to_string(), "宗教驱魔".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 28,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "冷静描述，渐进不安".to_string(),
            preferred_structures: vec![
                "档案体".to_string(),
                "调查报告".to_string(),
                "列表式".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "冷静精确，括号注释".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.03,
            preferred_devices: vec!["暗示".to_string(), "并列".to_string()],
            imagery_preference: vec![
                "建筑意象".to_string(),
                "档案意象".to_string(),
                "阈限意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.0,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.02,
            dominant_mood: "不安疏离".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.7,
            signature_patterns: vec![
                "官僚问答".to_string(),
                "录音转写".to_string(),
                "冷静报告".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 硬科幻风格
/// 特征：技术细节、概念密集、冷静、工程师思维
pub fn hard_sf() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "硬科幻".to_string(),
            author: None,
            description: "以严格科学为基础，技术细节密集，工程师思维，概念优先，冷静推演"
                .to_string(),
            genre_association: Some("硬科幻".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "futuristic".to_string(),
            preferred_categories: vec![
                "物理术语".to_string(),
                "工程词汇".to_string(),
                "数学概念".to_string(),
                "天文术语".to_string(),
            ],
            signature_words: vec![
                "轨道".to_string(),
                "引擎".to_string(),
                "辐射".to_string(),
                "计算".to_string(),
            ],
            avoided_patterns: vec!["情感铺陈".to_string(), "魔法元素".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 32,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "信息密集，逻辑推进".to_string(),
            preferred_structures: vec![
                "技术说明".to_string(),
                "推演论证".to_string(),
                "场景模拟".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "精确清晰，术语密集".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.01,
            preferred_devices: vec!["类比".to_string()],
            imagery_preference: vec!["科技意象".to_string(), "宇宙意象".to_string()],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.15,
            omniscience_level: 0.3,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.01,
            dominant_mood: "冷静理性".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "dry".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.2,
            signature_patterns: vec![
                "技术讨论".to_string(),
                "简报式".to_string(),
                "工程师幽默".to_string(),
            ],
            tag_style: "said_only".to_string(),
        },
    }
}

/// 史诗奇幻风格
/// 特征：托尔金式、宏大、神话、中古用语
pub fn epic_fantasy() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "史诗奇幻".to_string(),
            author: None,
            description: "托尔金式宏大奇幻，神话体系，中古氛围，善恶对抗，世界观详尽".to_string(),
            genre_association: Some("奇幻/史诗".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "中古词汇".to_string(),
                "神话术语".to_string(),
                "种族用语".to_string(),
                "魔法词汇".to_string(),
            ],
            signature_words: vec![
                "命运".to_string(),
                "王国".to_string(),
                "宝剑".to_string(),
                "龙".to_string(),
            ],
            avoided_patterns: vec!["现代科技".to_string(), "口语俚语".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 34,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "宏大庄严，史诗吟诵".to_string(),
            preferred_structures: vec![
                "史诗叙述".to_string(),
                "预言".to_string(),
                "种族语言".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "传统庄重，长句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.08,
            preferred_devices: vec!["比喻".to_string(), "象征".to_string(), "预言".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "神话意象".to_string(),
                "战争意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.8,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.06,
            dominant_mood: "庄严悲壮".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.3,
            signature_patterns: vec![
                "中古腔调".to_string(),
                "预言式".to_string(),
                "种族口音".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}
