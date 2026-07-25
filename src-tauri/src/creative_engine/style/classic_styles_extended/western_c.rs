use super::super::dna::*;

/// 狄更斯风格
/// 特征：社会批判、人物类型化、温情、连载节奏
pub fn dickens() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "狄更斯".to_string(),
            author: Some("查尔斯·狄更斯".to_string()),
            description: "维多利亚时代小说巨匠，以夸张生动的人物和社会批判写伦敦众生相，温情脉脉"
                .to_string(),
            genre_association: Some("社会批判/连载".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "伦敦方言".to_string(),
                "法律术语".to_string(),
                "贫困词汇".to_string(),
                "儿童用语".to_string(),
            ],
            signature_words: vec![
                "伦敦".to_string(),
                "雾".to_string(),
                "孤儿".to_string(),
                "圣诞".to_string(),
            ],
            avoided_patterns: vec!["粗俗直描".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 28,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "生动活泼，戏剧性".to_string(),
            preferred_structures: vec![
                "类型化描写".to_string(),
                "悬念结尾".to_string(),
                "温情转折".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "戏剧化，感叹号频繁".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.07,
            preferred_devices: vec!["夸张".to_string(), "拟人".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "城市意象".to_string(),
                "贫困意象".to_string(),
                "自然意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.15,
            omniscience_level: 0.9,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.07,
            dominant_mood: "温情悲悯".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.4,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.3,
            signature_patterns: vec![
                "方言腔调".to_string(),
                "戏剧式".to_string(),
                "温情说教".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 福楼拜风格
/// 特征：客观、精雕细琢、包法利式、农民语言
pub fn flaubert() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "福楼拜".to_string(),
            author: Some("居斯塔夫·福楼拜".to_string()),
            description: "法国现实主义巅峰，以极度客观和精雕细琢的笔法写人性欲望，作者隐退"
                .to_string(),
            genre_association: Some("现实主义/自然主义".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "医学词汇".to_string(),
                "农业术语".to_string(),
                "色彩词汇".to_string(),
                "宗教用语".to_string(),
            ],
            signature_words: vec![
                "包法利".to_string(),
                "外省".to_string(),
                "梦想".to_string(),
                "庸俗".to_string(),
            ],
            avoided_patterns: vec![
                "作者评论".to_string(),
                "道德判断".to_string(),
                "情感直白".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 36,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "精确冷静，如外科手术".to_string(),
            preferred_structures: vec![
                "场景描写".to_string(),
                "自由间接引语".to_string(),
                "细节堆砌".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "精确冷静，长句为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.05,
            preferred_devices: vec!["象征".to_string(), "对比".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "乡村意象".to_string(),
                "物质意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.25,
            omniscience_level: 0.4,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "冷静悲悯".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "自由间接引语".to_string(),
                "农民口语".to_string(),
                "社交寒暄".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 雨果风格
/// 特征：浪漫主义、宏大、人道、史诗
pub fn hugo() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "雨果".to_string(),
            author: Some("维克多·雨果".to_string()),
            description: "法国浪漫主义巨匠，以宏大的叙事和人道主义情怀写历史与社会，激情澎湃"
                .to_string(),
            genre_association: Some("浪漫主义/史诗".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "历史术语".to_string(),
                "建筑词汇".to_string(),
                "海洋词汇".to_string(),
                "宗教用语".to_string(),
            ],
            signature_words: vec![
                "人民".to_string(),
                "自由".to_string(),
                "苦难".to_string(),
                "光明".to_string(),
            ],
            avoided_patterns: vec!["平淡克制".to_string(), "琐碎日常".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 42,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "激情澎湃，排山倒海".to_string(),
            preferred_structures: vec![
                "长篇议论".to_string(),
                "全景描写".to_string(),
                "对比排比".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "感叹号、分号、破折号密集".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.09,
            preferred_devices: vec![
                "比喻".to_string(),
                "排比".to_string(),
                "对比".to_string(),
                "呼告".to_string(),
            ],
            imagery_preference: vec![
                "建筑意象".to_string(),
                "海洋意象".to_string(),
                "人民意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.95,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.08,
            dominant_mood: "激情人道".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "verbose".to_string(),
            subtext_ratio: 0.3,
            signature_patterns: vec![
                "宣言式".to_string(),
                "长篇辩论".to_string(),
                "戏剧式".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 纳博科夫风格
/// 特征：博学、文字游戏、华丽、不可靠叙事
pub fn nabokov() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "纳博科夫".to_string(),
            author: Some("弗拉基米尔·纳博科夫".to_string()),
            description: "俄裔美国文学大师，以博学和文字游戏构建华丽迷宫，不可靠叙事，语言炫技"
                .to_string(),
            genre_association: Some("后现代/元小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "多语言词汇".to_string(),
                "蝴蝶学术".to_string(),
                "象棋术语".to_string(),
                "文学典故".to_string(),
            ],
            signature_words: vec![
                "蝴蝶".to_string(),
                "洛丽塔".to_string(),
                "语言".to_string(),
                "记忆".to_string(),
            ],
            avoided_patterns: vec!["平淡直白".to_string(), "道德说教".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 44,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "华丽繁复，如蝴蝶振翅".to_string(),
            preferred_structures: vec![
                "长句嵌套".to_string(),
                "文字游戏".to_string(),
                "元叙事".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "精致繁复，括号注释".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.13,
            preferred_devices: vec![
                "比喻".to_string(),
                "双关".to_string(),
                "典故".to_string(),
                "戏仿".to_string(),
            ],
            imagery_preference: vec![
                "蝴蝶意象".to_string(),
                "童年意象".to_string(),
                "语言意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.55,
            omniscience_level: 0.0,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "智性迷狂".to_string(),
            emotional_arc_pattern: "cyclical".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.8,
            signature_patterns: vec![
                "多语言夹杂".to_string(),
                "文字游戏".to_string(),
                "不可靠叙述".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}
