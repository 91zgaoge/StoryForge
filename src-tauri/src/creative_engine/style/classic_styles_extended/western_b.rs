use super::super::dna::*;

/// 博尔赫斯风格
/// 特征：迷宫、智性、浓缩、时间循环、图书馆
pub fn borges() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "博尔赫斯".to_string(),
            author: Some("豪尔赫·路易斯·博尔赫斯".to_string()),
            description: "阿根廷文学大师，以智性迷宫和浓缩的笔法探索时间、无限与镜像，百科全书式"
                .to_string(),
            genre_association: Some("后现代/幻想".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "哲学术语".to_string(),
                "神学词汇".to_string(),
                "东方典故".to_string(),
                "数学术语".to_string(),
            ],
            signature_words: vec![
                "迷宫".to_string(),
                "镜子".to_string(),
                "无限".to_string(),
                "图书馆".to_string(),
            ],
            avoided_patterns: vec!["冗长描写".to_string(), "情感铺陈".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 28,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "浓缩精炼，如寓言".to_string(),
            preferred_structures: vec![
                "浓缩叙述".to_string(),
                "伪学术".to_string(),
                "循环结构".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "简洁精确，句号有力".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.08,
            preferred_devices: vec!["寓言".to_string(), "悖论".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "迷宫意象".to_string(),
                "镜子意象".to_string(),
                "时间意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.3,
            omniscience_level: 0.1,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.02,
            dominant_mood: "智性孤寂".to_string(),
            emotional_arc_pattern: "static".to_string(),
            humor_style: "dry".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.1,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.9,
            signature_patterns: vec![
                "哲学问答".to_string(),
                "箴言式".to_string(),
                "间接引语".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 科塔萨尔风格
/// 特征：日常变形、奇幻跳脱、游戏规则、读者参与
pub fn cortazar() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "科塔萨尔".to_string(),
            author: Some("胡里奥·科塔萨尔".to_string()),
            description: "拉美文学爆炸代表，以日常变形和跳脱结构打破叙事常规，游戏感，读者参与"
                .to_string(),
            genre_association: Some("后现代/奇幻".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "日常词汇".to_string(),
                "游戏术语".to_string(),
                "音乐术语".to_string(),
                "动物词汇".to_string(),
            ],
            signature_words: vec![
                "门".to_string(),
                "跳房子".to_string(),
                "兔子".to_string(),
                "地铁".to_string(),
            ],
            avoided_patterns: vec!["宏大叙事".to_string(), "道德说教".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 26,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "跳跃灵动，如爵士即兴".to_string(),
            preferred_structures: vec![
                "日常变形".to_string(),
                "分支叙事".to_string(),
                "读者指令".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "活泼，善用逗号与破折号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.07,
            preferred_devices: vec!["超现实".to_string(), "游戏".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "都市意象".to_string(),
                "动物意象".to_string(),
                "游戏意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.25,
            omniscience_level: 0.4,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: " playful 疏离".to_string(),
            emotional_arc_pattern: "cyclical".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "日常怪谈".to_string(),
                "游戏式".to_string(),
                "读者对话".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 爱伦·坡风格
/// 特征：哥特恐怖、韵律、死亡、心理分析
pub fn poe() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "爱伦·坡".to_string(),
            author: Some("埃德加·爱伦·坡".to_string()),
            description: "哥特文学之父，以精密计算的语言营造恐怖氛围，死亡迷恋，心理分析先驱"
                .to_string(),
            genre_association: Some("哥特/恐怖".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "死亡词汇".to_string(),
                "建筑术语".to_string(),
                "心理术语".to_string(),
                "色彩词汇".to_string(),
            ],
            signature_words: vec![
                "死亡".to_string(),
                "乌鸦".to_string(),
                "心脏".to_string(),
                "坟墓".to_string(),
            ],
            avoided_patterns: vec!["日常口语".to_string(), "幽默轻松".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 30,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "韵律感强，如诗歌".to_string(),
            preferred_structures: vec!["重复".to_string(), "递进".to_string(), "倒叙".to_string()],
            opening_variety: "moderate".to_string(),
            punctuation_style: "感叹号、破折号、分号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.1,
            preferred_devices: vec!["象征".to_string(), "重复".to_string(), "夸张".to_string()],
            imagery_preference: vec![
                "黑暗意象".to_string(),
                "死亡意象".to_string(),
                "建筑意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.55,
            omniscience_level: 0.0,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.1,
            dominant_mood: "恐怖阴郁".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.15,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "独白".to_string(),
                "疯狂低语".to_string(),
                "死亡宣告".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 洛夫克拉夫特风格
/// 特征：宇宙恐怖、不可名状、冗长、科学冷静
pub fn lovecraft() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "洛夫克拉夫特".to_string(),
            author: Some("H.P.洛夫克拉夫特".to_string()),
            description: "克苏鲁神话创始人，以科学冷静的长篇描写构建宇宙恐怖，不可名状，细节密集"
                .to_string(),
            genre_association: Some("宇宙恐怖/科幻".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "古词汇".to_string(),
                "科学术语".to_string(),
                "建筑术语".to_string(),
                "神话词汇".to_string(),
            ],
            signature_words: vec![
                "不可名状".to_string(),
                "疯狂".to_string(),
                "远古".to_string(),
                "深渊".to_string(),
            ],
            avoided_patterns: vec![
                "日常口语".to_string(),
                "幽默".to_string(),
                "情感直白".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 48,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "冗长密集，层层叠加".to_string(),
            preferred_structures: vec![
                "长篇描写".to_string(),
                "条件从句".to_string(),
                "否定式".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "逗号密集，长句连绵".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.08,
            preferred_devices: vec!["暗示".to_string(), "夸张".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "宇宙意象".to_string(),
                "建筑意象".to_string(),
                "深渊意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.4,
            omniscience_level: 0.0,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "宇宙恐怖".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.1,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.8,
            signature_patterns: vec![
                "警告".to_string(),
                "日记体".to_string(),
                "科学记录".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 简·奥斯汀风格
/// 特征：讽刺、礼仪、机智、婚姻市场
pub fn austen() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "简·奥斯汀".to_string(),
            author: Some("简·奥斯汀".to_string()),
            description: "英国古典讽刺大师，以机智优雅的笔法剖析婚姻与阶级，讽刺含蓄，对话精彩"
                .to_string(),
            genre_association: Some("社会风俗/浪漫".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "社交用语".to_string(),
                "财产词汇".to_string(),
                "礼仪用语".to_string(),
                "情感委婉语".to_string(),
            ],
            signature_words: vec![
                "婚姻".to_string(),
                "财产".to_string(),
                "体面".to_string(),
                "偏见".to_string(),
            ],
            avoided_patterns: vec!["粗俗口语".to_string(), "直白情感".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 34,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "优雅从容，如舞步".to_string(),
            preferred_structures: vec![
                "自由间接引语".to_string(),
                "反讽对比".to_string(),
                "礼貌迂回".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "精致，善用逗号分号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["反讽".to_string(), "对比".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "社交意象".to_string(),
                "乡村意象".to_string(),
                "财产意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.35,
            omniscience_level: 0.5,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "机智优雅".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.4,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.7,
            signature_patterns: vec![
                "机智交锋".to_string(),
                "礼貌刺探".to_string(),
                "间接表白".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}
