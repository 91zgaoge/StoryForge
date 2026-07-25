use super::super::dna::*;

/// 陀思妥耶夫斯基风格
/// 特征：心理深渊、癫狂、长独白、罪与罚
pub fn dostoevsky() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "陀思妥耶夫斯基".to_string(),
            author: Some("陀思妥耶夫斯基".to_string()),
            description: "俄罗斯文学深渊，以癫狂的长篇独白探索人性的罪恶与救赎，心理描写极致"
                .to_string(),
            genre_association: Some("心理小说/哲学".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "宗教术语".to_string(),
                "哲学词汇".to_string(),
                "心理术语".to_string(),
                "俄语语气".to_string(),
            ],
            signature_words: vec![
                "上帝".to_string(),
                "罪恶".to_string(),
                "疯狂".to_string(),
                "苦难".to_string(),
            ],
            avoided_patterns: vec!["简洁克制".to_string(), "平淡叙述".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 55,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "汹涌澎湃，一气呵成".to_string(),
            preferred_structures: vec![
                "长篇独白".to_string(),
                "对话辩论".to_string(),
                "意识流".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "感叹号、破折号、分号密集".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.09,
            preferred_devices: vec!["反问".to_string(), "夸张".to_string(), "对比".to_string()],
            imagery_preference: vec![
                "宗教意象".to_string(),
                "黑暗意象".to_string(),
                "城市意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.6,
            omniscience_level: 0.4,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.1,
            dominant_mood: "狂热绝望".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.4,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.3,
            signature_patterns: vec![
                "长篇辩论".to_string(),
                "癫狂独白".to_string(),
                "哲学质问".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 托尔斯泰风格
/// 特征：史诗全景、道德、朴素、历史与家庭
pub fn tolstoy() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "托尔斯泰".to_string(),
            author: Some("列夫·托尔斯泰".to_string()),
            description: "俄国文学泰斗，以史诗般的全景视角写历史与家庭，道德追问，朴素而深邃"
                .to_string(),
            genre_association: Some("史诗/现实主义".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "农事词汇".to_string(),
                "军事术语".to_string(),
                "宗教词汇".to_string(),
                "家庭用语".to_string(),
            ],
            signature_words: vec![
                "灵魂".to_string(),
                "土地".to_string(),
                "战争".to_string(),
                "和平".to_string(),
            ],
            avoided_patterns: vec!["华丽修饰".to_string(), "过度象征".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 40,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "雄浑开阔，从容不迫".to_string(),
            preferred_structures: vec![
                "全景描写".to_string(),
                "内心分析".to_string(),
                "历史议论".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "清晰从容，长句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.05,
            preferred_devices: vec!["对比".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "战争意象".to_string(),
                "家庭意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.95,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.05,
            dominant_mood: "悲悯庄严".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "家庭辩论".to_string(),
                "内心独白式对话".to_string(),
                "俄国贵族腔".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 卡夫卡风格
/// 特征：荒诞、异化、冷静恐怖、官僚迷宫
pub fn kafka() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "卡夫卡".to_string(),
            author: Some("弗朗茨·卡夫卡".to_string()),
            description: "现代主义先驱，以冷静理性的笔法写荒诞与异化，官僚迷宫，存在焦虑"
                .to_string(),
            genre_association: Some("现代主义/荒诞".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "法律术语".to_string(),
                "官僚用语".to_string(),
                "建筑词汇".to_string(),
                "家庭称谓".to_string(),
            ],
            signature_words: vec![
                "审判".to_string(),
                "变形".to_string(),
                "门".to_string(),
                "城堡".to_string(),
            ],
            avoided_patterns: vec!["情感词汇".to_string(), "抒情表达".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 35,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "冷静冗长，近乎报告".to_string(),
            preferred_structures: vec![
                "长句铺排".to_string(),
                "条件从句".to_string(),
                "间接引语".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "冷静精确，逗号连接".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.03,
            preferred_devices: vec!["象征".to_string(), "寓言".to_string()],
            imagery_preference: vec![
                "建筑意象".to_string(),
                "迷宫意象".to_string(),
                "变形意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.4,
            omniscience_level: 0.1,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.02,
            dominant_mood: "焦虑荒诞".to_string(),
            emotional_arc_pattern: "static".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "官僚式".to_string(),
                "荒诞问答".to_string(),
                "间接引语".to_string(),
            ],
            tag_style: "said_only".to_string(),
        },
    }
}

/// 福克纳风格
/// 特征：美国南方、多角度、繁复长句、时间跳跃
pub fn faulkner() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "福克纳".to_string(),
            author: Some("威廉·福克纳".to_string()),
            description: "美国南方文学代表，以繁复长句和多角度叙事写家族史诗，时间跳跃，意识流"
                .to_string(),
            genre_association: Some("南方哥特/现代主义".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "南方方言".to_string(),
                "家族词汇".to_string(),
                "自然词汇".to_string(),
                "宗教用语".to_string(),
            ],
            signature_words: vec![
                "时间".to_string(),
                "家族".to_string(),
                "土地".to_string(),
                "荣誉".to_string(),
            ],
            avoided_patterns: vec!["简洁克制".to_string(), "线性叙事".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 65,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "奔腾不息，意识流淌".to_string(),
            preferred_structures: vec![
                "长句嵌套".to_string(),
                "意识流".to_string(),
                "时间跳跃".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "括号、分号、破折号繁复".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.1,
            preferred_devices: vec![
                "象征".to_string(),
                "意识流".to_string(),
                "多角度".to_string(),
            ],
            imagery_preference: vec![
                "南方意象".to_string(),
                "家族意象".to_string(),
                "自然意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.5,
            omniscience_level: 0.6,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.08,
            dominant_mood: "悲怆狂乱".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "南方口音".to_string(),
                "家族 gossip".to_string(),
                "意识流对话".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 菲茨杰拉德风格
/// 特征：爵士时代、华丽忧郁、美国梦、精致感伤
pub fn fitzgerald() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "菲茨杰拉德".to_string(),
            author: Some("菲茨杰拉德".to_string()),
            description: "爵士时代代言人，以华丽精致的文风写美国梦的破灭，忧郁感伤，金句频出"
                .to_string(),
            genre_association: Some("爵士时代/现代主义".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "奢华词汇".to_string(),
                "色彩词汇".to_string(),
                "音乐术语".to_string(),
                "时代用语".to_string(),
            ],
            signature_words: vec![
                "绿灯".to_string(),
                "梦想".to_string(),
                "奢华".to_string(),
                "失落".to_string(),
            ],
            avoided_patterns: vec!["粗俗口语".to_string(), "直白议论".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 32,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "华丽流畅，诗化散文".to_string(),
            preferred_structures: vec![
                "意象铺陈".to_string(),
                "对比".to_string(),
                "象征".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "精致，善用分号与破折号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.12,
            preferred_devices: vec!["比喻".to_string(), "象征".to_string(), "对比".to_string()],
            imagery_preference: vec![
                "奢华意象".to_string(),
                "色彩意象".to_string(),
                "梦想意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.35,
            omniscience_level: 0.1,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.07,
            dominant_mood: "华丽忧伤".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "dry".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.35,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "机智交谈".to_string(),
                "社交寒暄".to_string(),
                "酒后真言".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}
