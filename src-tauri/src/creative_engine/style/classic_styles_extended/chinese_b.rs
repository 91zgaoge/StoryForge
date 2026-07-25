use super::super::dna::*;

/// 王小波风格
/// 特征：幽默反讽、理性思辨、自由洒脱、口语化智慧
pub fn wang_xiaobo() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "王小波".to_string(),
            author: Some("王小波".to_string()),
            description: "特立独行的作家，以幽默反讽包裹理性思辨，文字洒脱自由，充满智慧光芒"
                .to_string(),
            genre_association: Some("当代小说/杂文".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "科学术语".to_string(),
                "性隐喻".to_string(),
                "黑色幽默".to_string(),
                "逻辑词汇".to_string(),
            ],
            signature_words: vec![
                "有趣".to_string(),
                "智慧".to_string(),
                "自由".to_string(),
                "荒诞".to_string(),
            ],
            avoided_patterns: vec![
                "道学口吻".to_string(),
                "权威腔调".to_string(),
                "悲情叙事".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 28,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "跳跃活泼，旁征博引".to_string(),
            preferred_structures: vec![
                "口语化议论".to_string(),
                "故事套故事".to_string(),
                "反讽对比".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "轻松自然，善用破折号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.08,
            preferred_devices: vec!["反讽".to_string(), "夸张".to_string(), "比喻".to_string()],
            imagery_preference: vec![
                "荒诞意象".to_string(),
                "性意象".to_string(),
                "科学意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.35,
            omniscience_level: 0.1,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "幽默清醒".to_string(),
            emotional_arc_pattern: "cyclical".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "机智问答".to_string(),
                "自嘲".to_string(),
                "逻辑诡辩".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 曹雪芹风格
/// 特征：诗词融合、细腻入微、贵族生活、宿命感
pub fn cao_xueqin() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "曹雪芹".to_string(),
            author: Some("曹雪芹".to_string()),
            description: "《红楼梦》作者，以诗词化的语言描绘贵族生活，细腻入微，人物众多而各具神态"
                .to_string(),
            genre_association: Some("古典世情".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "诗词典故".to_string(),
                "园林词汇".to_string(),
                "服饰器物".to_string(),
                "人情世故".to_string(),
            ],
            signature_words: vec![
                "花落".to_string(),
                "梦".to_string(),
                "情".to_string(),
                "空".to_string(),
            ],
            avoided_patterns: vec!["粗俗口语".to_string(), "直白议论".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 36,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "骈散结合，诗词化节奏".to_string(),
            preferred_structures: vec![
                "对偶".to_string(),
                "排比".to_string(),
                "诗词嵌入".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "传统标点，善用逗号延伸".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.11,
            preferred_devices: vec!["隐喻".to_string(), "象征".to_string(), "对仗".to_string()],
            imagery_preference: vec![
                "花卉意象".to_string(),
                "梦境意象".to_string(),
                "器物意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.25,
            omniscience_level: 0.85,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.06,
            dominant_mood: "繁华悲凉".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.35,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "机锋往来".to_string(),
                "诗词对答".to_string(),
                "笑语藏针".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 蒲松龄风格
/// 特征：文言志怪、诡谲幽微、善恶报应、简洁传神
pub fn pu_songling() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "蒲松龄".to_string(),
            author: Some("蒲松龄".to_string()),
            description: "《聊斋志异》作者，以简练文言写鬼神狐妖，诡谲幽微，善恶分明，余韵悠长"
                .to_string(),
            genre_association: Some("文言志怪".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "文言词汇".to_string(),
                "鬼神术语".to_string(),
                "草木鸟兽".to_string(),
            ],
            signature_words: vec![
                "狐".to_string(),
                "鬼".to_string(),
                "异".to_string(),
                "怪".to_string(),
            ],
            avoided_patterns: vec!["白话俗语".to_string(), "冗长铺陈".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 18,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "文言短句，戛然而止".to_string(),
            preferred_structures: vec![
                "史传笔法".to_string(),
                "四字格".to_string(),
                "省略主语".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "句读简洁，句号密集".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["拟人".to_string(), "象征".to_string(), "暗示".to_string()],
            imagery_preference: vec![
                "幽冥意象".to_string(),
                "自然意象".to_string(),
                "幻化意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.05,
            omniscience_level: 0.9,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "幽微诡谲".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.7,
            signature_patterns: vec![
                "文言对白".to_string(),
                "寓意式对话".to_string(),
                "画龙点睛".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 苏轼风格
/// 特征：豪放洒脱、古文功底、旷达人生观、兼融儒释道
pub fn su_shi() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "苏轼".to_string(),
            author: Some("苏轼".to_string()),
            description: "北宋文豪，文风豪放洒脱，诗词文赋皆精，旷达通透，议论风生".to_string(),
            genre_association: Some("古典散文/词".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "典故".to_string(),
                "自然意象".to_string(),
                "哲理词汇".to_string(),
                "饮食词汇".to_string(),
            ],
            signature_words: vec![
                "明月".to_string(),
                "大江".to_string(),
                "浮生".to_string(),
                "旷达".to_string(),
            ],
            avoided_patterns: vec!["矫揉造作".to_string(), "悲戚过度".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 24,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "疏朗开阔，跌宕有致".to_string(),
            preferred_structures: vec![
                "散文化长句".to_string(),
                "议论排比".to_string(),
                "情景交融".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "传统标点，舒展自然".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.08,
            preferred_devices: vec!["比喻".to_string(), "用典".to_string(), "对仗".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "历史意象".to_string(),
                "人生意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.35,
            omniscience_level: 0.2,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.05,
            dominant_mood: "旷达洒脱".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.15,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "哲理问答".to_string(),
                "典故引用".to_string(),
                "旷达之语".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 阿城风格
/// 特征：古典白描、淡泊节制、智慧内敛、棋道人生
pub fn a_cheng() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "阿城".to_string(),
            author: Some("阿城".to_string()),
            description: "当代作家，以极度克制的白描笔法写知青生活，古典韵味，淡泊中见深邃"
                .to_string(),
            genre_association: Some("当代小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "器物词汇".to_string(),
                "棋道术语".to_string(),
                "饮食词汇".to_string(),
                "古典白描".to_string(),
            ],
            signature_words: vec![
                "棋".to_string(),
                "树".to_string(),
                "吃".to_string(),
                "闲".to_string(),
            ],
            avoided_patterns: vec![
                "抒情议论".to_string(),
                "心理分析".to_string(),
                "形容词堆砌".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 19,
            clause_complexity: "simple".to_string(),
            rhythm_pattern: "冲淡平和，近乎古人笔记".to_string(),
            preferred_structures: vec![
                "白描".to_string(),
                "短句".to_string(),
                "动作先行".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "极简，句号为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.02,
            preferred_devices: vec!["白描".to_string()],
            imagery_preference: vec!["日常意象".to_string(), "器物意象".to_string()],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.05,
            omniscience_level: 0.3,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.02,
            dominant_mood: "淡泊通透".to_string(),
            emotional_arc_pattern: "static".to_string(),
            humor_style: "dry".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "简短有力".to_string(),
                "动作伴随".to_string(),
                "留白".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}
