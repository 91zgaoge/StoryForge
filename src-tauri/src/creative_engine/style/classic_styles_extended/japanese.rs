use super::super::dna::*;

/// 川端康成风格
/// 特征：物哀、新感觉派、纤细、色彩与季节
pub fn kawabata_yasunari() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "川端康成".to_string(),
            author: Some("川端康成".to_string()),
            description: "诺贝尔文学奖得主，以纤细敏感的笔触捕捉日本美学的精髓，物哀、幽玄、余情"
                .to_string(),
            genre_association: Some("新感觉派/纯文学".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "色彩词汇".to_string(),
                "季节用语".to_string(),
                "身体感知".to_string(),
                "传统美学".to_string(),
            ],
            signature_words: vec![
                "雪".to_string(),
                "花".to_string(),
                "夜".to_string(),
                "镜".to_string(),
            ],
            avoided_patterns: vec!["直白议论".to_string(), "冗长描写".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 24,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "纤细流畅，余韵悠长".to_string(),
            preferred_structures: vec![
                "意象并置".to_string(),
                "省略主语".to_string(),
                "季节前置".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "柔和，句尾余韵".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.11,
            preferred_devices: vec!["通感".to_string(), "象征".to_string(), "暗示".to_string()],
            imagery_preference: vec![
                "季节意象".to_string(),
                "色彩意象".to_string(),
                "身体意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.35,
            omniscience_level: 0.2,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "物哀幽玄".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.8,
            signature_patterns: vec![
                "沉默间隙".to_string(),
                "含蓄试探".to_string(),
                "未尽之言".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 三岛由纪夫风格
/// 特征：暴烈美学、古典华丽、肌肉与死亡、仪式感
pub fn mishima_yukio() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "三岛由纪夫".to_string(),
            author: Some("三岛由纪夫".to_string()),
            description: "日本战后文学异端，以暴烈华丽的语言追求美与死亡的极致融合，古典与现代交织"
                .to_string(),
            genre_association: Some("后现代/美学小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "balanced".to_string(),
            temporal_quality: "mixed".to_string(),
            preferred_categories: vec![
                "古典词汇".to_string(),
                "身体词汇".to_string(),
                "色彩词汇".to_string(),
                "军事术语".to_string(),
            ],
            signature_words: vec![
                "太阳".to_string(),
                "肌肉".to_string(),
                "血".to_string(),
                "金阁".to_string(),
            ],
            avoided_patterns: vec!["平淡口语".to_string(), "日常琐事".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 38,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "激昂与静谧强烈对比".to_string(),
            preferred_structures: vec![
                "长句铺排".to_string(),
                "仪式感描写".to_string(),
                "对比并列".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "繁复精致，句号与感叹号交替".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.13,
            preferred_devices: vec![
                "比喻".to_string(),
                "象征".to_string(),
                "夸张".to_string(),
                "对偶".to_string(),
            ],
            imagery_preference: vec![
                "身体意象".to_string(),
                "太阳意象".to_string(),
                "死亡意象".to_string(),
            ],
            parallelism_frequency: "frequent".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "close_third".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.45,
            omniscience_level: 0.3,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.08,
            dominant_mood: "暴烈唯美".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.6,
            signature_patterns: vec![
                "宣言式".to_string(),
                "古典敬语".to_string(),
                "激烈独白".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 太宰治风格
/// 特征：斜阳、颓废、自毁、软弱与讨好
pub fn dazai_osamu() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "太宰治".to_string(),
            author: Some("太宰治".to_string()),
            description: "无赖派代表，以自毁式的坦诚写人类的软弱与羞耻，语气讨好又绝望".to_string(),
            genre_association: Some("无赖派/私小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "自贬词汇".to_string(),
                "死亡意象".to_string(),
                "酒类词汇".to_string(),
                "女性称谓".to_string(),
            ],
            signature_words: vec![
                "羞耻".to_string(),
                "失败".to_string(),
                "酒".to_string(),
                "女人".to_string(),
            ],
            avoided_patterns: vec!["自信表达".to_string(), "成功叙事".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 30,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "絮叨自白，断续彷徨".to_string(),
            preferred_structures: vec![
                "独白".to_string(),
                "自贬句式".to_string(),
                "反复道歉".to_string(),
            ],
            opening_variety: "repetitive".to_string(),
            punctuation_style: "省略号、破折号频繁".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["自嘲".to_string(), "反讽".to_string()],
            imagery_preference: vec![
                "黑暗意象".to_string(),
                "堕落意象".to_string(),
                "女性意象".to_string(),
            ],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "intimate".to_string(),
            interior_monologue_ratio: 0.65,
            omniscience_level: 0.0,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "expressive".to_string(),
            emotion_word_density: 0.09,
            dominant_mood: "颓废自毁".to_string(),
            emotional_arc_pattern: "cyclical".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "讨好语气".to_string(),
                "自我贬低".to_string(),
                "玩笑掩饰".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 夏目漱石风格
/// 特征：余裕派、知识分子、幽默、心理深潜
pub fn natsume_soseki() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "夏目漱石".to_string(),
            author: Some("夏目漱石".to_string()),
            description: "日本近代文学巨擘，以知识分子的视角剖析现代人的孤独与利己，余裕派美学"
                .to_string(),
            genre_association: Some("近代文学/知识小说".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "学术术语".to_string(),
                "自然意象".to_string(),
                "心理词汇".to_string(),
                "汉文调".to_string(),
            ],
            signature_words: vec![
                "孤独".to_string(),
                "余裕".to_string(),
                "月亮".to_string(),
                "猫".to_string(),
            ],
            avoided_patterns: vec!["通俗口语".to_string(), " melodramatic 表达".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 42,
            clause_complexity: "complex".to_string(),
            rhythm_pattern: "从容不迫，汉文调余韵".to_string(),
            preferred_structures: vec![
                "长句议论".to_string(),
                "心理分析".to_string(),
                "迂回表达".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "传统与现代融合，分号破折号".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.07,
            preferred_devices: vec!["比喻".to_string(), "反讽".to_string(), "象征".to_string()],
            imagery_preference: vec![
                "自然意象".to_string(),
                "知识意象".to_string(),
                "孤独意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "first_person".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.5,
            omniscience_level: 0.0,
            temporal_handling: "linear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.04,
            dominant_mood: "孤独余裕".to_string(),
            emotional_arc_pattern: "gradual".to_string(),
            humor_style: "witty".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.25,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.7,
            signature_patterns: vec![
                "知识分子腔".to_string(),
                "迂回试探".to_string(),
                "自嘲式".to_string(),
            ],
            tag_style: "action_beats".to_string(),
        },
    }
}

/// 芥川龙之介风格
/// 特征：历史题材、冷峻怀疑、精致短篇、人性黑暗
pub fn akutagawa_ryunosuke() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "芥川龙之介".to_string(),
            author: Some("芥川龙之介".to_string()),
            description: "短篇小说鬼才，以冷峻精致的笔法重写历史题材，怀疑主义，人性黑暗面"
                .to_string(),
            genre_association: Some("历史小说/短篇".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "high".to_string(),
            abstraction: "abstract".to_string(),
            temporal_quality: "archaic".to_string(),
            preferred_categories: vec![
                "历史用语".to_string(),
                "佛教术语".to_string(),
                "古典词汇".to_string(),
                "病态词汇".to_string(),
            ],
            signature_words: vec![
                "罗生门".to_string(),
                "疑惑".to_string(),
                "利己".to_string(),
                "地狱".to_string(),
            ],
            avoided_patterns: vec!["温情脉脉".to_string(), "道德说教".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 30,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "冷峻精致，如刀削木雕".to_string(),
            preferred_structures: vec![
                "史传笔法".to_string(),
                "多角度叙述".to_string(),
                "悬念结尾".to_string(),
            ],
            opening_variety: "varied".to_string(),
            punctuation_style: "简洁精确，句号有力".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.06,
            preferred_devices: vec!["反讽".to_string(), "象征".to_string(), "暗示".to_string()],
            imagery_preference: vec![
                "历史意象".to_string(),
                "黑暗意象".to_string(),
                "宗教意象".to_string(),
            ],
            parallelism_frequency: "moderate".to_string(),
            irony_usage: "overt".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "distant".to_string(),
            interior_monologue_ratio: 0.15,
            omniscience_level: 0.6,
            temporal_handling: "flashback".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "冷峻怀疑".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "dark".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.2,
            dialogue_length: "terse".to_string(),
            subtext_ratio: 0.8,
            signature_patterns: vec![
                "古典白话".to_string(),
                "冷嘲".to_string(),
                "沉默".to_string(),
            ],
            tag_style: "minimal".to_string(),
        },
    }
}

/// 东野圭吾风格
/// 特征：社会派推理、冷静、反转、日常恐怖
pub fn higashino_keigo() -> StyleDNA {
    StyleDNA {
        meta: StyleMeta {
            name: "东野圭吾".to_string(),
            author: Some("东野圭吾".to_string()),
            description: "日本推理天王，以冷静克制的笔法写社会派推理，反转精妙，人性剖析深刻"
                .to_string(),
            genre_association: Some("社会派推理".to_string()),
        },
        vocabulary: VocabularyProfile {
            density: "medium".to_string(),
            abstraction: "concrete".to_string(),
            temporal_quality: "modern".to_string(),
            preferred_categories: vec![
                "日常词汇".to_string(),
                "科技词汇".to_string(),
                "法律术语".to_string(),
                "心理术语".to_string(),
            ],
            signature_words: vec![
                "真相".to_string(),
                "动机".to_string(),
                "秘密".to_string(),
                "绝望".to_string(),
            ],
            avoided_patterns: vec!["华丽辞藻".to_string(), "过度抒情".to_string()],
        },
        syntax: SyntaxProfile {
            avg_sentence_length: 22,
            clause_complexity: "moderate".to_string(),
            rhythm_pattern: "冷静推进，信息密集".to_string(),
            preferred_structures: vec![
                "调查推进".to_string(),
                "多线并行".to_string(),
                "时间跳跃".to_string(),
            ],
            opening_variety: "moderate".to_string(),
            punctuation_style: "清晰冷静，句号为主".to_string(),
        },
        rhetoric: RhetoricProfile {
            metaphor_density: 0.02,
            preferred_devices: vec!["伏笔".to_string(), "暗示".to_string()],
            imagery_preference: vec!["日常意象".to_string(), "科技意象".to_string()],
            parallelism_frequency: "rare".to_string(),
            irony_usage: "subtle".to_string(),
        },
        perspective: PerspectiveProfile {
            pov_type: "multiple".to_string(),
            narrative_distance: "close".to_string(),
            interior_monologue_ratio: 0.2,
            omniscience_level: 0.5,
            temporal_handling: "nonlinear".to_string(),
        },
        emotion: EmotionProfile {
            expressiveness: "restrained".to_string(),
            emotion_word_density: 0.03,
            dominant_mood: "冷静绝望".to_string(),
            emotional_arc_pattern: "sudden".to_string(),
            humor_style: "none".to_string(),
        },
        dialogue: DialogueProfile {
            dialogue_ratio: 0.35,
            dialogue_length: "moderate".to_string(),
            subtext_ratio: 0.5,
            signature_patterns: vec![
                "审讯式".to_string(),
                "试探性".to_string(),
                "关键线索".to_string(),
            ],
            tag_style: "said_only".to_string(),
        },
    }
}
