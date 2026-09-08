use crate::constant::meta::{MARKDOWN_BASE_TITLE_TAG, MARKDOWN_PREFIX_TITLE_LIST};
use crate::util::error::AppError;
use pulldown_cmark::utils::TextMergeStream;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{debug, error, warn};

// 从 markdown 文档中解析出题目结构和内容
// 1. 目前表格处理只能处理固定的表格, 存在其余表格时无法正确解析, 需后续完善
// 2. 图片不支持, 因为文档本身无法提供图片

// 原始题目内容
#[derive(Debug)]
pub struct RawQuestion {
    pub level: String,                // 分层体系
    pub title: String,                // 标题
    pub choices: Vec<(char, String)>, // 选项内容
    pub question_type: String,        // 题目类型
    pub dimensions: Vec<String>,      // 核心素养
    pub knowledge: Vec<String>,       // 知识点
    pub difficulty_level: String,     // 难度
    pub scenes: Vec<String>,          // 适用场景
    pub mistake_tips: Vec<String>,    // 常见错误
    pub answer: String,               // 参考答案
    pub analysis: String,             // 解题分析
    pub detail: String,               // 详解对应解题过程
}

#[derive(Debug)]
pub struct Question {
    pub parent: RawQuestion,        // 母题
    pub children: Vec<RawQuestion>, // 变式题列表
}

// 一级：分母题
// 按母题分块, 区块内的变式题均为母题的变式题
fn get_parents(text: &str) -> Vec<String> {
    let mut parents = Vec::new();
    let mut buf = String::new();
    let mut started = false;
    for line in text.lines() {
        if line.trim_start().starts_with(MARKDOWN_BASE_TITLE_TAG) {
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

// 二级：分所有H4 H5标题（母题+变式）
// 返回 标题->原始整体内容 的列表, 仅仅是根据标题分组, 原始内容全部保留
fn get_parent_and_children(block: &str) -> Vec<(String, String)> {
    let mut res = Vec::new();
    let mut buf = String::new();
    let mut level = String::new();

    for line in block.lines() {
        let is_match = MARKDOWN_PREFIX_TITLE_LIST
            .iter()
            .any(|p| line.trim_start().starts_with(p));
        if is_match {
            // 第二次遇见标题行说明一个题目已完整记录到 buf, 保存后清空继续处理下一个题
            if !buf.is_empty() {
                res.push((level.clone(), buf.trim().to_string()));
                buf.clear();
            }

            let trimmed_line = line.trim_start();
            if let Some(matched_prefix) = MARKDOWN_PREFIX_TITLE_LIST
                .iter()
                .find(|&&p| trimmed_line.starts_with(p))
            {
                level = trimmed_line[matched_prefix.len()..].trim().to_string();
            }
        }
        // 将当前行也追加到原始内容中
        if !line.trim().is_empty() || is_match {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if !buf.trim().is_empty() {
        res.push((level.clone(), buf.trim().to_string()));
    }
    res
}

// 按照 --- 分隔符将变式题拆分
fn get_children(children: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut current_block = String::new();

    for line in children.lines() {
        let trimmed = line.trim();

        // 基础语法 如果遇到标准的分割线
        if trimmed == "---" {
            // 如果当前题目缓冲区有内容 说明一题结束
            if !current_block.trim().is_empty() {
                res.push(current_block.trim().to_string());
                current_block.clear();
            }
        } else {
            // 否则 继续把当前行和换行符追加到这一题的内容里
            // 过滤掉完全没用的纯空白行 保持内容紧凑
            if !line.is_empty() || !current_block.is_empty() {
                current_block.push_str(line);
                current_block.push('\n');
            }
        }
    }

    // 收尾工作 最后一道题后面通常没有 ---
    if !current_block.trim().is_empty() {
        res.push(current_block.trim().to_string());
    }

    res
}

// 记录每个标签节点
#[derive(PartialEq, Debug)]
enum Section {
    None,
    Title,             // 题干
    Choices,           // 选项
    QuestionType,      // 题目类型
    QuestionDimension, // 核心素养
    Knowledge,         // 知识点
    DifficultyLevel,   // 难度
    Scenes,            // 适用场景
    MistakeTips,       // 常见错误
    Answer,            // 参考答案
    Analysis,          // 解题分析
    Detail,            // 解题过程
}

// 主Markdown->结构化题的解析
// 原始 Markdown（以及 CommonMark、GFM 等主流实现）的核心规则
// 行末两个空格 + 换行符 → 产生 <br> 换行（软换行）
// 单纯的换行符 → 被当作普通空格处理，不会产生新行（即相邻两行会合并成一行）
fn parse_question(level: String, markdown: &str) -> Result<RawQuestion, AppError> {
    let parser = Parser::new(markdown);
    let events = TextMergeStream::new(parser);

    // 记录标签事件
    let mut state = Section::None;

    // 保存关注的内容
    let mut title: String = String::new();
    let mut choice: String = String::new();
    let mut question_type: String = String::new();
    let mut dimensions: Vec<String> = Vec::new();
    let mut knowledge: Vec<String> = Vec::new();
    let mut difficulty_level: String = String::new();
    let mut scenes: Vec<String> = Vec::new();
    let mut mistake_tips: Vec<String> = Vec::new();
    let mut answer: String = String::new();
    let mut analysis: String = String::new();
    let mut detail: String = String::new();

    // 是否是加粗的文本
    let mut in_strong = false;

    for event in events {
        match event {
            Event::Start(Tag::Strong) => {
                in_strong = true;
            }
            Event::End(TagEnd::Strong) => {
                in_strong = false;
            }
            Event::Text(t) => {
                let s = t.trim();
                if s.is_empty() {
                    continue;
                }

                // 去掉前后的中英文冒号和空格
                let clean_s = s
                    .trim_start_matches([':', '：'])
                    .trim_end_matches([':', '：'])
                    .trim();
                if clean_s.is_empty() {
                    continue;
                }

                // 精准匹配 Markdown 文本关键字
                if in_strong {
                    match clean_s {
                        // 因为变式题格式是 **1. 题目**
                        _ if clean_s.ends_with("题目") => {
                            state = Section::Title;
                            continue;
                        }
                        "选项" => {
                            state = Section::Choices;
                            continue;
                        }
                        "题目类型" => {
                            state = Section::QuestionType;
                            continue;
                        }
                        "核心素养" => {
                            state = Section::QuestionDimension;
                            continue;
                        }
                        "知识点" => {
                            state = Section::Knowledge;
                            continue;
                        }
                        "难度" => {
                            state = Section::DifficultyLevel;
                            continue;
                        }
                        "适用场景" => {
                            state = Section::Scenes;
                            continue;
                        }
                        "常见错误" => {
                            state = Section::MistakeTips;
                            continue;
                        }
                        "参考答案" => {
                            state = Section::Answer;
                            continue;
                        }
                        "思路介绍" => {
                            state = Section::Analysis;
                            continue;
                        }
                        "详细解析" => {
                            state = Section::Detail;
                            continue;
                        }
                        _ => {
                            warn!("Unknown event encountered clean_s: {}", clean_s);
                            state = Section::None;
                        }
                    }
                }

                match state {
                    Section::Title => title.push_str(clean_s),
                    Section::Choices => choice.push_str(clean_s),
                    Section::QuestionType => question_type.push_str(clean_s),
                    Section::DifficultyLevel => difficulty_level.push_str(clean_s),
                    Section::QuestionDimension
                    | Section::Knowledge
                    | Section::Scenes
                    | Section::MistakeTips => {
                        let target_vec = match state {
                            Section::QuestionDimension => &mut dimensions,
                            Section::Knowledge => &mut knowledge,
                            Section::Scenes => &mut scenes,
                            _ => &mut mistake_tips,
                        };
                        for item in clean_s.split(['、', '，', ',']) {
                            if !item.trim().is_empty() {
                                target_vec.push(item.trim().to_string());
                            }
                        }
                    }
                    Section::Answer => {
                        answer.push_str(clean_s);
                        answer.push_str("  \n");
                    }
                    Section::Analysis => {
                        analysis.push_str(clean_s);
                        analysis.push_str("  \n");
                    }
                    Section::Detail => {
                        detail.push_str(clean_s);
                        detail.push_str("  \n");
                    }
                    Section::None => {
                        warn!("Unknown section state: {:?}, clean_s: {}", state, clean_s);
                    }
                }
            }
            _ => {}
        }
    }

    // 解析选项, 没有选项的题目为空
    let choices = if choice.is_empty() {
        vec![]
    } else {
        get_choices(&choice)?
    };

    let raw = RawQuestion {
        level,
        title,
        choices,
        question_type,
        dimensions,
        knowledge,
        difficulty_level,
        scenes,
        mistake_tips,
        answer,
        analysis,
        detail,
    };

    Ok(raw)
}

// 解析出题目难度, 解析失败等均返回 1
pub fn get_difficulty_level(val: &str) -> Decimal {
    // 允许的分数集合使用 Decimal
    const ALLOWED: [&str; 9] = ["1", "1.5", "2", "2.5", "3", "3.5", "4", "4.5", "5"];

    // 解析为 Decimal
    let num = Decimal::from_str(val.trim()).unwrap_or_else(|_| Decimal::from(1));

    // 检查是否在允许列表中（通过字符串比较或转为字符串后比较）
    let num_str = num.to_string();
    if ALLOWED.contains(&num_str.as_str()) {
        num
    } else {
        Decimal::from(1)
    }
}

// 解析出选项列表
pub fn get_choices(text: &str) -> Result<Vec<(char, String)>, AppError> {
    if text.is_empty() {
        return Err(AppError::param_error("选项内容为空"));
    }

    // 这个正则极其简单，只负责精准寻找选项的开头（如 "A." 或 "B．"）
    let re =
        Regex::new(r"([A-D])[.．]").map_err(|_| AppError::param_error("选项匹配正则生成失败"))?;

    // 找出所有匹配项的起点和终点
    let matches: Vec<_> = re.find_iter(text).collect();
    if matches.is_empty() {
        error!("parse md choice text: {}", text);
        return Err(AppError::param_error("没有解析出有效的选项"));
    }

    let mut result: Vec<(char, String)> = Vec::new();

    for i in 0..matches.len() {
        let current_match = &matches[i];

        // 提取当前的选项字母 (A, B, C, D)
        // 因为匹配到的是 "A." 或 "A．"，第一个字符必然是字母
        let letter = current_match.as_str().chars().next().unwrap_or('A');

        // 核心逻辑：当前选项内容的起点是当前匹配的结束位置
        let start_pos = current_match.end();

        // 终点是下一个选项的起点；如果是最后一个选项，则一直到文本末尾
        let end_pos = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            text.len()
        };

        // 安全地进行字符串切片，并清洗两端多余的空格（包含全角空格 '　'）
        if let Some(raw_content) = text.get(start_pos..end_pos) {
            let content = raw_content
                .trim_matches(|c: char| c.is_whitespace() || c == '　')
                .to_string();
            if !content.is_empty() {
                result.push((letter, content));
            }
        }
    }

    if result.is_empty() {
        error!("parse choices text to result: {}", text);
        return Err(AppError::param_error("拆分选项标识和内容后选项内容为空"));
    }

    // 按字母顺序排序（A, B, C, D）
    result.sort_by_key(|(letter, _)| *letter);

    Ok(result)
}

// 得到所有的问题列表
pub fn get_questions(content: &str) -> Result<Vec<Question>, AppError> {
    let blocks = get_parents(content);
    let mut all_questions = Vec::new();

    for block in blocks {
        let subs = get_parent_and_children(&block);
        // 没有母题变式题
        if subs.is_empty() {
            warn!("split parents and children subs is empty");
            continue;
        }
        // 第一项为母题
        let (parent_level, parent_md) = &subs[0];
        debug!("parent level: {}", parent_level);
        let parent_struct = parse_question(parent_level.clone(), parent_md)?;

        // 变式列表拆分
        let mut similars = Vec::new();
        for (child_level, child_md) in subs.iter().skip(1) {
            debug!("child level: {}", child_level);
            let children = get_children(child_md);
            if children.is_empty() {
                warn!("split children subs is empty");
                continue;
            }
            for child in children {
                let var = parse_question(child_level.clone(), &child)?;
                similars.push(var);
            }
        }

        all_questions.push(Question {
            parent: parent_struct,
            children: similars,
        });
    }

    Ok(all_questions)
}

// 将一段 markdown 片段尝试解析出一个题目
pub fn get_question(content: &str) -> Result<RawQuestion, AppError> {
    parse_question("".to_string(), content)
}

#[cfg(test)]
mod tests {
    use crate::util::markdown::get_questions;

    #[test]
    fn test_parse() {
        let content = r#"
**题库说明**

- 适用版本：2024湖南教育出版社（湘教版）
- 分层体系：母题（广西中考经典考法）→ 基础变式 → 提升变式 → 应用变式（场景变式）→ 拓展变式
- 题量标准：每个核心知识点配置3-9道母题，每道母题下四个维度各3道变式题
- 母题来源：每道母题均为该知识点广西近年中考经典考题或课本重要考题改编
- 题目配套：每道题标注**「题目」「题目类型」「核心素养」「知识点」「难度星级」**，附**「参考答案」「思路介绍」「详细解析」**（以上信息加粗，后面的配文不加粗）
- 难度说明：1 基础题　2 中档题　3 较难题　4 难题　5 压轴题
- 适用场景：每道题标注1-3个适用教学场景，包括课前预习、随堂练习、课后作业、复习巩固、单元检测、错题补漏、中考复习
- 常见错误：每道题标注1-3个学生易犯错误类型，包括概念不清、理解偏差、审题不清、条件遗漏、计算失误、符号书写错误、公式不熟、方法不会、不会迁移应用、答题规范/格式错误

---

# 1.1 认识负数

## S-1.1.1.1 相反意义的量的表示

### 母题1：正负数表示相反意义的量

**题目：** 将海平面的高度记作 0 m，珠穆朗玛峰峰顶高出海平面 8848.86 m，"奋斗者"号载人潜水器坐底深度低于海平面 10909 m，分别用正数和负数表示这两个高度。

**题目类型：** 解答题

**核心素养：** 数感

**知识点：** 相反意义的量、正负数的意义

**难度：** 1

**适用场景：** 课前预习、随堂练习

**常见错误：** 概念不清、审题不清

**参考答案：** 珠穆朗玛峰高度记作 +8848.86 m；奋斗者号坐底深度记作 -10909 m

**思路介绍：** 规定海平面为 0 基准，高于海平面的量用正数表示，低于海平面的量用负数表示。

**详细解析：** 以海平面为分界点，高出海平面的高度为正，因此珠穆朗玛峰高度记为 +8848.86 m；低于海平面的深度为负，因此坐底深度记为 -10909 m。

---

#### 基础变式

**1. 题目：** 如果节约 20 m³ 水记作 +20 m³，那么浪费 10 m³ 水记作 ______

**选项：** A. -9 m³　B. -11 m³　C. 10 m³　D. -10 m³

**题目类型：** 选择题

**核心素养：** 数感

**知识点：** 相反意义的量

**难度：** 1

**适用场景：** 课前预习、随堂练习

**常见错误：** 概念不清、审题不清

**参考答案：** D

**思路介绍：** 根据知识点分析各选项，正确答案为 D。

**详细解析：** 题目规定节约用正数表示，与节约相反的浪费则用负数表示，因此浪费 10 m³ 记作 -10 m³。

---

**2. 题目：** 如果 -30 万元表示亏损 30 万元，那么 +20 万元表示 ______

**选项：** A. 无法确定　B. 盈利 20 万元　C. 0　D. 以上都不对

**题目类型：** 选择题

**核心素养：** 数感、应用意识

**知识点：** 相反意义的量

**难度：** 1

**适用场景：** 课前预习、随堂练习

**常见错误：** 概念不清、审题不清

**参考答案：** B

**思路介绍：** 根据知识点分析各选项，正确答案为 B。

**详细解析：** -30 万元对应亏损，说明负数表示亏损，因此正数 +20 万元表示盈利 20 万元。

---

**3. 题目：** 如果向东走 50 米记作 +50 米，那么向西走 30 米记作 ______，原地不动记作 ______。

**题目类型：** 填空题

**核心素养：** 数感

**知识点：** 相反意义的量

**难度：** 1

**适用场景：** 课前预习、随堂练习

**常见错误：** 概念不清、审题不清

**参考答案：** -30 米；0 米

**思路介绍：** 向东为正，向西为负，基准位置为 0。

**详细解析：** 向东与向西是相反意义的量，向东记正则向西记负，因此向西走 30 米记作 -30 米；原地不动是基准点，记作 0 米。

---

#### 提升变式

**1. 题目：** 某药品说明书上标明药品保存温度是 (20±2)℃，该药品在什么温度范围内保存才合适？

**题目类型：** 解答题

**核心素养：** 数感、应用意识

**知识点：** 正负数的实际意义、相反意义的量

**难度：** 2

**适用场景：** 随堂练习、课后作业

**常见错误：** 条件遗漏、概念不清、审题不清

**参考答案：** 18 ℃ ~ 22 ℃

**思路介绍：** 20±2 ℃ 表示以 20 ℃ 为标准，上下浮动 2 ℃。

**详细解析：** +2 ℃ 表示比 20 ℃ 高 2 ℃，即 22 ℃；-2 ℃ 表示比 20 ℃ 低 2 ℃，即 18 ℃。因此合适的保存温度范围是 18 ℃ 到 22 ℃。

---

**2. 题目：** 数学测验班级平均分为 85 分，高于平均分记为正，小明得 92 分记作 +7 分，小红得 81 分记作 ______

**选项：** A. -5 分　B. -3 分　C. 4 分　D. -4 分

**题目类型：** 选择题

**核心素养：** 数感、应用意识

**知识点：** 相反意义的量、基准数

**难度：** 2

**适用场景：** 随堂练习、课后作业

**常见错误：** 计算失误、概念不清、审题不清

**参考答案：** D

**思路介绍：** 根据知识点分析各选项，正确答案为 D。

**详细解析：** 81 分比平均分 85 分低 4 分，按照规则低于平均分记为负，因此记作 -4 分。

---

**3. 题目：** 某仓库运进面粉 7.5 吨记作 +7.5 吨，那么先运进 3.2 吨，再运出 5.6 吨，最终库存变化记作 ______

**选项：** A. -1.4 吨　B. -3.4 吨　C. 2.4 吨　D. -2.4 吨

**题目类型：** 选择题

**核心素养：** 数感、应用意识

**知识点：** 相反意义的量、有理数加减意义

**难度：** 2

**适用场景：** 随堂练习、课后作业

**常见错误：** 概念不清、审题不清

**参考答案：** D

**思路介绍：** 根据知识点分析各选项，正确答案为 D。

**详细解析：** 运进 3.2 吨记为 +3.2 吨，运出 5.6 吨记为 -5.6 吨，最终变化为 3.2 + (-5.6) = -2.4 吨，即库存减少 2.4 吨。

---
        "#;
        let all_questions = get_questions(content);

        // 输出结构
        for parent in all_questions.unwrap() {
            println!("\n=== 分层体系：{} ===", parent.parent.level);
            println!("题目: {}", parent.parent.title);
            println!("选项: {:?}", parent.parent.choices);
            println!("题目类型: {:?}", parent.parent.question_type);
            println!("核心素养: {:?}", parent.parent.dimensions);
            println!("知识点: {:?}", parent.parent.knowledge);
            println!("难度: {:?}", parent.parent.difficulty_level);
            println!("适用场景: {:?}", parent.parent.scenes);
            println!("常见错误: {:?}", parent.parent.mistake_tips);
            println!("参考答案: {}", parent.parent.answer);
            println!("思路介绍: {}", parent.parent.analysis);
            println!("详细解析: {}", parent.parent.detail);
            for v in &parent.children {
                println!("  -- 变式题分层体系：{}", v.level);
                println!("     题目: {}", v.title);
                println!("     选项: {:?}", v.choices);
                println!("     题目类型: {:?}", v.question_type);
                println!("     核心素养: {:?}", v.dimensions);
                println!("     知识点: {:?}", v.knowledge);
                println!("     难度: {:?}", v.difficulty_level);
                println!("     适用场景: {:?}", v.scenes);
                println!("     常见错误: {:?}", v.mistake_tips);
                println!("     参考答案: {}", v.answer);
                println!("     思路介绍: {}", v.analysis);
                println!("     详细解析: {}", v.detail);
            }
        }
    }
}
