use super::super::dna::*;

/// 鲁迅风格
/// 特征：冷峻犀利、白话文运动、讽刺、解剖国民性
pub fn lu_xun() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "鲁迅".to_string(),
            author: Some("鲁迅".to_string()),
            description: "现代文学奠基人，以冷峻犀利的笔触解剖国民性，讽刺辛辣，白描精准，\
                          情感深沉内敛"
                .to_string(),
            genre_association: Some("现实主义/杂文".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "医学隐喻".to_string(),
                "解剖术语".to_string(),
                "冷峻白描".to_string(),
                "讽刺语汇".to_string(),
            ],
            signature_words: vec![
                "铁屋子".to_string(),
                "看客".to_string(),
                "麻木".to_string(),
                "脊梁".to_string(),
            ],
            avoided_patterns: vec![
                "华丽辞藻".to_string(),
                "温情脉脉".to_string(),
                "说教口吻".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 32,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "冷峻短句与长句交替，顿挫感强".to_string(),
            preferred_structures: vec![
                "白描".to_string(),
                "反讽".to_string(),
                "递进式质问".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "善用句号制造停顿，省略号留白".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.07,
            preferred_devices: vec!["反讽".to_string(), "象征".to_string(), "隐喻".to_string()],
            imagery_preference: vec![
                "病态意象".to_string(),
                "铁屋意象".to_string(),
                "暗夜意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.1,
            omniscience_level: 0.4,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "悲愤沉郁".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.8,
            signature_patterns: vec![
                "话中有刺".to_string(),
                "沉默胜过言语".to_string(),
                "方言土语夹杂".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 老舍风格
/// 特征：京味儿、市井烟火、幽默温厚、口语化
pub fn lao_she() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "老舍".to_string(),
            author: Some("老舍".to_string()),
            description: "京味文学大师，以温厚幽默的笔触描绘市井生活，口语鲜活，人物栩栩如生"
                .to_string(),
            genre_association: Some("京味文学/市民小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "北京方言".to_string(),
                "市井俗语".to_string(),
                "饮食词汇".to_string(),
                "行当术语".to_string(),
            ],
            signature_words: vec![
                "咱".to_string(),
                "得嘞".to_string(),
                "劳驾".to_string(),
                "人缘儿".to_string(),
            ],
            avoided_patterns: vec!["书面雅语".to_string(), "欧化长句".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 22,
            clause_complexity: "simple".to_string(),
            rhythm_pattern: "口语化流畅，如听评书".to_string(),
            preferred_structures: vec![
                "对话推进".to_string(),
                "短句连缀".to_string(),
                "俗语入文".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "逗号句号为主，贴近口语停顿".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.04,
            preferred_devices: vec!["拟人".to_string(), "反讽".to_string()],
            imagery_preference: vec![
                "市井意象".to_string(),
                "饮食意象".to_string(),
                "季节意象".to_string(),
            ],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.15,
            omniscience_level: 0.8,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "balanced".to_string(),
            emotion_word_density: 0.05,
            dominant_mood: "温厚悲悯".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.45,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.3,
            signature_patterns: vec![
                "京片子".to_string(),
                "俏皮话".to_string(),
                "儿化音".to_string(),
            ],
            tag_style: "varied_tags".to_string(),
        },
    }
}

/// 沈从文风格
/// 特征：湘西风情、田园牧歌、清澈自然、抒情诗化
pub fn shen_congwen() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "沈从文".to_string(),
            author: Some("沈从文".to_string()),
            description: "湘西世界的歌者，以清澈如水的文字描绘边地风情，诗化叙事，人性纯美"
                .to_string(),
            genre_association: Some("乡土抒情/牧歌".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "自然词汇".to_string(),
                "湘西方言".to_string(),
                "色彩词汇".to_string(),
                "水意象".to_string(),
            ],
            signature_words: vec![
                "渡船".to_string(),
                "吊脚楼".to_string(),
                "流水".to_string(),
                "山歌".to_string(),
            ],
            avoided_patterns: vec![
                "城市词汇".to_string(),
                "现代术语".to_string(),
                "抽象议论".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 26,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "舒缓流畅，如水波荡漾".to_string(),
            preferred_structures: vec![
                "景物铺陈".to_string(),
                "长短句交错".to_string(),
                "民歌化句式".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "柔和，善用逗号延伸".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.09,
            preferred_devices: vec!["比喻".to_string(), "拟人".to_string(), "通感".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "水意象".to_string(),
                "乡土意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "none".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "omniscient".to_string(),
            narrative_distance: "moderate".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.7,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "清澈忧伤".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.4,
            signature_patterns: vec![
                "山歌对唱".to_string(),
                "含蓄表白".to_string(),
                "方言轻柔".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 余华风格
/// 特征：冷酷叙事、暴力美学、简朴直白、黑色幽默
pub fn yu_hua() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "余华".to_string(),
            author: Some("余华".to_string()),
            description: "先锋文学代表，以冷酷直白的笔触直面暴力与死亡，后期转向温情但底色苍凉"
                .to_string(),
            genre_association: Some("先锋文学/现实主义".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "low".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "身体词汇".to_string(),
                "死亡意象".to_string(),
                "日常词汇".to_string(),
            ],
            signature_words: vec![
                "活着".to_string(),
                "血".to_string(),
                "死亡".to_string(),
                "忍受".to_string(),
            ],
            avoided_patterns: vec![
                "华丽辞藻".to_string(),
                "心理分析".to_string(),
                "抒情议论".to_string(),
            ],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 20,
            clause_complexity: "simple".to_string(),
            rhythm_pattern: "冷静克制，近乎医学报告".to_string(),
            preferred_structures: vec![
                "主谓宾直叙".to_string(),
                "并列短句".to_string(),
                "重复句式".to_string(),
            ],
            opening_variety: "repetitive".to_string(),
            punctuation_style: "极简，句号为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.02,
            preferred_devices: vec!["象征".to_string()],
            imagery_preference: vec!["身体意象".to_string(), "死亡意象".to_string()],
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
            dominant_mood: "苍凉宿命".to_string(),
            emotional_arc_pattern: "static".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.3,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.2,
            signature_patterns: vec![
                "直白粗粝".to_string(),
                "重复确认".to_string(),
                "沉默间隙".to_string(),
            ],
            tag_style: "said_only".to_string(),
        },
    }
}
