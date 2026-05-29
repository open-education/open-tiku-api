use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use regex::Regex;
use rust_decimal::Decimal;
use std::io::Error;
use std::str::FromStr;

/// 从 markdown 文档中解析出题目结构和内容
/// 1. 目前表格处理只能处理固定的表格, 存在其余表格时无法正确解析, 需后续完善
/// 2. 图片不支持, 因为文档本身无法提供图片

// 原始题目内容
#[derive(Debug)]
pub struct RawQuestion {
    pub title: String,            // 标题
    pub stem: String,             // 题干
    pub difficulty_level: String, // 难度
    pub stage: String,            // 学段
    pub question_type: String,    // 题目类型
    pub choices: Vec<String>,     // 选项内容
    pub knowledge: String,        // 知识点
    pub answer: String,           // 参考答案
    pub analysis: String,         // 解题分析
    pub detail: String,           // 详解, 对应解题过程
}

#[derive(Debug)]
pub struct Question {
    pub parent: RawQuestion,        // 母题
    pub children: Vec<RawQuestion>, // 变式题列表
}

// 一级：分母题
// 按母题分块, 区块内的变式题均为母题的变式题
fn split_parents(text: &str) -> Vec<String> {
    let mut parents = Vec::new();
    let mut buf = String::new();
    let mut started = false;
    for line in text.lines() {
        if line.trim_start().starts_with("##### 母题") {
            // 遇到第二个母题将上一个母题记录并清空 buf 继续保存其它题目
            if started && !buf.trim().is_empty() {
                parents.push(buf.trim().to_string());
                buf.clear();
            }
            started = true;
        }
        if started {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if !buf.trim().is_empty() {
        parents.push(buf.trim().to_string());
    }
    parents
}

// 二级：分所有H5标题（母题+变式）
// 返回 标题->原始整体内容 的列表
fn split_parents_and_children(block: &str) -> Vec<(String, String)> {
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut title = String::new();
    for line in block.lines() {
        if line.trim_start().starts_with("##### ") {
            // 第二次遇见标题行说明一个题目已完整记录到 buf, 保存后清空继续处理下一个题
            if !buf.is_empty() {
                res.push((title.clone(), buf.trim().to_string()));
                buf.clear();
            }
            title = line
                .trim_start()
                .trim_start_matches("#####")
                .trim()
                .to_string();
        }
        // 将当前行也追加到原始内容中
        if !line.trim().is_empty() || line.trim_start().starts_with("##### ") {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if !buf.trim().is_empty() {
        res.push((title.clone(), buf.trim().to_string()));
    }
    res
}

// 题干&选项切割正则，兼容全角半角
fn extract_choices_and_stem(text: &str, question_type: &str) -> (String, Vec<String>) {
    if !question_type.eq("选择题") {
        return (text.to_string(), Vec::new());
    }

    let re = Regex::new(r"[A-D][.．][^A-D　\n]+").unwrap();
    let choices: Vec<String> = re
        .find_iter(text)
        .map(|m| m.as_str().trim().to_string())
        .collect();

    let stem = re.replace_all(text, "").to_string();
    let stem = stem
        .trim()
        .replace("（  　）", "（    ）")
        .trim()
        .to_string();
    (stem, choices)
}

// 记录每个标签节点
#[derive(PartialEq, Debug)]
enum Section {
    None,
    Head5,           // 母题变式题标题
    DifficultyLevel, // 难度
    Stage,           // 学段
    QuestionType,    // 题目类型
    Knowledge,       // 知识点
    Answer,          // 参考答案
    Analysis,        // 解题分析
    Detail,          // 解题过程, 详解
}

// 主Markdown->结构化题的解析
fn parse_question(title: String, markdown: &str) -> RawQuestion {
    let parser = Parser::new(markdown);

    let mut state = Section::None;

    let mut head5 = String::new(); // 母题变式题标题
    let mut main_content = String::new(); // 题目主体内容包括选项等
    let mut difficulty_level = String::new(); // 难度
    let mut stage = String::new(); // 学段
    let mut question_type = String::new(); // 题目类型
    let mut knowledge = String::new(); // 知识点
    let mut answer = String::new(); // 参考答案
    let mut analysis = String::new(); // 解题分析
    let mut detail = String::new(); // 解题过程-详解
    let mut in_strong = false; // 是否在后续的几个加粗标签中

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if level == pulldown_cmark::HeadingLevel::H5 {
                    state = Section::Head5;
                }
            }
            Event::End(TagEnd::Heading { .. }) => {
                if state == Section::Head5 {
                    state = Section::None;
                }
            }
            Event::Start(Tag::Strong) => {
                in_strong = true;
            }
            Event::End(TagEnd::Strong) => {
                in_strong = false;
            }
            Event::Text(t) => {
                let s = t.trim();

                // 加粗的类型
                if in_strong {
                    if s == "难度：" {
                        state = Section::DifficultyLevel;
                        continue;
                    } else if s == "适用学期：" {
                        state = Section::Stage;
                        continue;
                    } else if s == "题目类型：" {
                        state = Section::QuestionType;
                        continue;
                    } else if s == "涉及知识点：" {
                        state = Section::Knowledge;
                        continue;
                    } else if s == "参考答案：" {
                        state = Section::Answer;
                        continue;
                    } else if s == "【分析】" {
                        state = Section::Analysis;
                        continue;
                    } else if s == "【详解】" {
                        state = Section::Detail;
                        continue;
                    }
                }
                match state {
                    Section::Head5 => head5.push_str(s),
                    Section::DifficultyLevel => difficulty_level.push_str(&s),
                    Section::Stage => stage.push_str(&s),
                    Section::QuestionType => question_type.push_str(&s),
                    Section::Knowledge => knowledge.push_str(s),
                    Section::Answer => answer.push_str(s),
                    Section::Analysis => analysis.push_str(s),
                    Section::Detail => detail.push_str(s),
                    Section::None => {
                        main_content.push_str(s);
                    }
                }
            }
            _ => {}
        }
    }
    let (stem, choices) = extract_choices_and_stem(&main_content, &question_type);

    RawQuestion {
        title: if title.is_empty() {
            head5.trim().to_string()
        } else {
            title
        },
        stem,
        choices,
        difficulty_level: difficulty_level.trim().to_string(),
        stage: stage.trim().to_string(),
        question_type: question_type.trim().to_string(),
        knowledge: knowledge.trim().to_string(),
        answer: answer.trim().to_string(),
        analysis: analysis.trim().to_string(),
        detail: detail.trim().to_string(),
    }
}

// 解析出题目难度, 解析失败等均返回 1
pub fn get_difficulty_level(table: &Vec<Vec<String>>) -> Decimal {
    // 允许的分数集合（使用 Decimal）
    const ALLOWED: [&str; 9] = ["1", "1.5", "2", "2.5", "3", "3.5", "4", "4.5", "5"];

    // 获取第一个单元格
    let first_cell = match table.first().and_then(|row| row.first()) {
        Some(s) => s,
        None => return Decimal::from(1),
    };

    // 提取【】中的内容
    let content = first_cell
        .strip_prefix('【')
        .and_then(|s| s.strip_suffix('】'))
        .unwrap_or("");

    // 解析为 Decimal
    let num = Decimal::from_str(content.trim()).unwrap_or_else(|_| Decimal::from(1));

    // 检查是否在允许列表中（通过字符串比较或转为字符串后比较）
    let num_str = num.to_string();
    if ALLOWED.contains(&num_str.as_str()) {
        num
    } else {
        Decimal::from(1)
    }
}

// 解析出选项列表
pub fn get_choices(choices: Vec<String>) -> Vec<(char, String)> {
    let mut result: Vec<(char, String)> = choices
        .into_iter()
        .filter_map(|s| {
            // 取第一个字符作为选项字母
            let mut chars = s.chars();
            let letter = chars.next()?;
            // 跳过点分隔符（可能是 '．' 或 '.'）
            let rest = chars.as_str().trim_start_matches(|c| c == '．' || c == '.');
            if rest.is_empty() {
                None
            } else {
                Some((letter, rest.to_string()))
            }
        })
        .collect();
    // 按字母顺序排序（A, B, C, D）
    result.sort_by_key(|(letter, _)| *letter);
    result
}

// 得到所有的问题列表
pub fn get_questions(content: &str) -> Result<Vec<Question>, Error> {
    let blocks = split_parents(content);
    let mut all_questions = Vec::new();

    for block in blocks {
        let subs = split_parents_and_children(&block);
        // 没有母题变式题
        if subs.is_empty() {
            continue;
        }
        // 第一项为母题
        let (parent_title, parent_md) = &subs[0];
        let parent_struct = parse_question(parent_title.clone(), parent_md);

        // 变式
        let mut var_vec = Vec::new();
        for (title, md) in subs.iter().skip(1) {
            let var = parse_question(title.clone(), md);
            var_vec.push(var);
        }

        all_questions.push(Question {
            parent: parent_struct,
            children: var_vec,
        });
    }

    Ok(all_questions)
}

#[cfg(test)]
mod tests {
    use crate::util::markdown_parse::{get_questions};

    #[test]
    fn test_parse() {
        let content = r#"##### 母题 1

当 $x = 2$ 时，代数式 $2x + 1$ 的值是（   ）

A．3　　B．5　　C．4　　D．6

** 难度：** 1

** 适用学期：** 71

** 题目类型：** 选择题

**涉及知识点：** 【求代数式的值】

**参考答案：** 【B】

**【分析】** 本题考查了代数式求值。将 $x = 2$ 代入代数式 $2x + 1$ 直接计算即可。

**【详解】** 解：∵ $x = 2$，
∴ $2x + 1 = 2 \times 2 + 1 = 4 + 1 = 5$。
故选：B。

---

##### 变式 1

当 $x = -1$ 时，代数式 $2x - 2$ 的值为（   ）

A．2　　B．$-2$　　C．4　　D．$-4$

** 难度：** 1

** 适用学期：** 71

** 题目类型：** 选择题

**涉及知识点：** 【求代数式的值】

**参考答案：** 【D】

**【分析】** 本题考查了求代数式的值，将 $x = -1$ 代入所求代数式计算即可得解。

**【详解】** 解：当 $x = -1$ 时，代数式 $2x - 2 = 2 \times (-1) - 2 = -2 - 2 = -4$。
故选：D。

---

##### 变式 3

有一列数 $a_{1},a_{2},a_{3},\ldots,a_{n}$， 其中 $a_{1} = 5 \times 2 + 1$，$a_{2} = 5 \times 3 + 2$，$a_{3} = 5 \times 4 + 3$，$a_{4} = 5 \times 5 + 4$，则 $a_{10} =$ \_\_\_\_\_\_ ，当 $a_{n} = 2021$ 时，$n =$ \_\_\_\_\_\_ 。

** 难度：** 1

** 适用学期：** 71

** 题目类型：** 选择题

**涉及知识点：** 【求代数式的值，整体求值】

**参考答案：** 【65，336】

**【分析】** 本题考查了规律问题。通过观察数列前几项的结构，可知 $a_{n} = 5(n + 1) + n = 6n + 5$，进而计算即可。

**【详解】** 解：$a_{1} = 5 \times 2 + 1$，
$a_{2} = 5 \times 3 + 2$，
$a_{3} = 5 \times 4 + 3$，
$a_{4} = 5 \times 5 + 4$，
......
$a_{n} = 5(n + 1) + n = 6n + 5$，
$a_{10} = 6 \times 10 + 5 = 65$，
当 $a_{n} = 2021$ 时，$6n + 5 = 2021$，解得：$n = 336$。
故答案为：65，336。

---"#;
        let all_questions = get_questions(content);

        // 输出结构
        for mother in all_questions.unwrap_or_default() {
            println!("\n=== 标题：{} ===", mother.parent.title);
            println!("题干: {}", mother.parent.stem);
            println!("选项: {:?}", mother.parent.choices);
            println!("难度: {:?}", mother.parent.difficulty_level);
            println!("适用学期: {:?}", mother.parent.stage);
            println!("题目类型: {:?}", mother.parent.question_type);
            println!("参考答案: {}", mother.parent.answer);
            println!("知识点: {}", mother.parent.knowledge);
            println!("分析: {}", mother.parent.analysis);
            println!("详解: {}", mother.parent.detail);
            for v in &mother.children {
                println!("  -- 变式标题：{}", v.title);
                println!("     题干: {}", v.stem);
                println!("     选项: {:?}", v.choices);
                println!("     难度: {:?}", v.difficulty_level);
                println!("     试用学期: {:?}", v.stage);
                println!("     题目类型: {:?}", v.question_type);
                println!("     参考答案: {}", v.answer);
                println!("     知识点: {}", v.knowledge);
                println!("     分析: {}", v.analysis);
                println!("     详解: {}", v.detail);
            }
        }
    }
}
