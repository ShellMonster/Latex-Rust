//! 常见数学函数命令映射，将 `\sin` 等指令渲染为文本节点

use phf::phf_map;

static FUNCTIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "sin" => "sin",
    "cos" => "cos",
    "tan" => "tan",
    "sec" => "sec",
    "csc" => "csc",
    "cot" => "cot",
    "sinh" => "sinh",
    "cosh" => "cosh",
    "tanh" => "tanh",
    "csch" => "csch",
    "sech" => "sech",
    "coth" => "coth",
    "arcsin" => "arcsin",
    "arccos" => "arccos",
    "arctan" => "arctan",
    "arccsc" => "arccsc",
    "arcsec" => "arcsec",
    "arccot" => "arccot",
    "arsinh" => "arsinh",
    "arcosh" => "arcosh",
    "artanh" => "artanh",
    "arcsch" => "arcsch",
    "arsech" => "arsech",
    "arcoth" => "arcoth",
    "log" => "log",
    "ln" => "ln",
    "exp" => "exp",
    "det" => "det",
    "ker" => "ker",
    "dim" => "dim",
    "sup" => "sup",
    "inf" => "inf",
    "lim" => "lim",
    "max" => "max",
    "min" => "min",
};

pub fn map_text_command(command: &str) -> Option<&'static str> {
    FUNCTIONS.get(command).copied()
}
