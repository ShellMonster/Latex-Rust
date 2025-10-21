//! 间距命令映射，将 `\,` 等转换为实际的 Unicode 空白字符

use phf::phf_map;

const THIN_SPACE: &str = "\u{2009}";
const MEDIUM_SPACE: &str = "\u{2004}";
const THICK_SPACE: &str = "\u{2005}";
const HAIR_SPACE: &str = "\u{200A}";
const EM_SPACE: &str = "\u{2003}";
const EN_SPACE: &str = "\u{2002}";

static SPACING: phf::Map<&'static str, &'static str> = phf_map! {
    "," => THIN_SPACE,
    ";" => THICK_SPACE,
    ":" => MEDIUM_SPACE,
    "!" => HAIR_SPACE,
    "quad" => EM_SPACE,
    "qquad" => "\u{2003}\u{2003}",
    " " => EN_SPACE,
};

pub fn map_spacing(command: &str) -> Option<&'static str> {
    SPACING.get(command).copied()
}
