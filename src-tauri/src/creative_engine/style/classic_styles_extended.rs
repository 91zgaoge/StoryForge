//! 经典风格扩展库
//!
//! 新增 40 位经典作家与类型文学风格 DNA，覆盖中/日/欧美文学及类型 fiction。
//! 与 classic_styles.rs 合并后总数达 52 种。
//!
//! 本文件为聚合入口，具体风格定义拆分到 classic_styles_extended/ 子模块中。

pub mod chinese_a;
pub mod chinese_b;
pub mod chinese_c;
pub mod genre_a;
pub mod genre_b;
pub mod japanese;
pub mod western_a;
pub mod western_b;
pub mod western_c;

pub use chinese_a::*;
pub use chinese_b::*;
pub use chinese_c::*;
pub use genre_a::*;
pub use genre_b::*;
pub use japanese::*;
pub use western_a::*;
pub use western_b::*;
pub use western_c::*;

use super::dna::StyleDNA;

/// 获取所有扩展的经典风格（40种）
pub fn get_extended_styles() -> Vec<StyleDNA> {
    vec![
        // 中国文学（12种）
        lu_xun(),
        lao_she(),
        shen_congwen(),
        yu_hua(),
        wang_xiaobo(),
        cao_xueqin(),
        pu_songling(),
        su_shi(),
        a_cheng(),
        bai_xianyong(),
        qian_zhongshu(),
        yu_dafu(),
        // 日本文学（6种）
        kawabata_yasunari(),
        mishima_yukio(),
        dazai_osamu(),
        natsume_soseki(),
        akutagawa_ryunosuke(),
        higashino_keigo(),
        // 欧美文学（14种）
        dostoevsky(),
        tolstoy(),
        kafka(),
        faulkner(),
        fitzgerald(),
        borges(),
        cortazar(),
        poe(),
        lovecraft(),
        austen(),
        dickens(),
        flaubert(),
        hugo(),
        nabokov(),
        // 类型文学（8种）
        cyberpunk(),
        steampunk(),
        new_weird(),
        hard_sf(),
        epic_fantasy(),
        grimdark(),
        xianxia(),
        infinite_flow(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_styles_count() {
        let styles = get_extended_styles();
        assert_eq!(styles.len(), 40);
    }

    #[test]
    fn test_lu_xun_dna() {
        let dna = lu_xun();
        assert_eq!(dna.meta.name, "鲁迅");
        assert_eq!(dna.emotion.expressiveness, "restrained");
    }

    #[test]
    fn test_dostoevsky_dna() {
        let dna = dostoevsky();
        assert_eq!(dna.meta.name, "陀思妥耶夫斯基");
        assert!(dna.syntax.avg_sentence_length > 50);
    }

    #[test]
    fn test_cyberpunk_dna() {
        let dna = cyberpunk();
        assert_eq!(dna.meta.name, "赛博朋克");
        assert_eq!(dna.vocabulary.temporal_quality, "futuristic");
    }
}
