-- 说明： IDE 格式化如果混乱, 可以关闭格式化, 用其它工具格式化, 真扯淡
--
-- 1. 创建教材层级表
CREATE TABLE IF NOT EXISTS textbook
(
    id         SERIAL PRIMARY KEY,
    path_type  VARCHAR(30)  NOT NULL DEFAULT 'common',             -- 路径类型 common 公共节点 knowledge 考点选题 chapter 章节选题
    parent_id  INTEGER REFERENCES textbook (id) ON DELETE CASCADE, -- 父级标识, 查询时需要指定 path_depth 控制深度
    label      VARCHAR(255) NOT NULL,                              -- 名称对应学段科目等
    key        VARCHAR(120) NOT NULL,                              -- 名称标识
    path_depth INTEGER,                                            -- 层级深度
    sort_order INTEGER               DEFAULT 0,                    -- 排序
    created_at TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);

-- 唯一索引：父级目录下的名称是唯一的, 跨层级不限制
CREATE UNIQUE INDEX IF NOT EXISTS uni_idx_parent_label ON textbook (parent_id, label);
-- 普通索引：深度查询优化
CREATE INDEX IF NOT EXISTS idx_textbook_depth ON textbook (path_depth);

-- 2. 章节小节和知识点关联
CREATE TABLE IF NOT EXISTS chapter_knowledge
(
    id           SERIAL PRIMARY KEY,                                  -- 自增主键
    chapter_id   INTEGER NOT NULL,                                    -- 教材章节id
    knowledge_id INTEGER NOT NULL,                                    -- 知识点分类id
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,               -- 自动写入创建时间
    CONSTRAINT uq_chapter_knowledge UNIQUE (chapter_id, knowledge_id) -- 约束：chapter_id + knowledge_id 必须唯一
);

-- 为单个 knowledge_id 查询创建索引 (chapter_id 作为联合唯一索引的第一列通常已自带索引)
CREATE INDEX IF NOT EXISTS idx_knowledge_id ON chapter_knowledge (knowledge_id);

-- 3. 题型列表存储
CREATE TABLE IF NOT EXISTS question_cate
(
    id         SERIAL PRIMARY KEY,
    related_id INTEGER REFERENCES chapter_knowledge (id) ON DELETE CASCADE, -- 关联标识
    label      VARCHAR(255) NOT NULL,                                       -- 题型名称
    key        VARCHAR(120) NOT NULL,                                       -- 题型标识
    sort_order INTEGER     DEFAULT 0,                                       -- 排序
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 关联标识创建普通索引
CREATE INDEX IF NOT EXISTS idx_related_id ON question_cate (related_id);

-- 4. 题型相关的字典
CREATE TABLE IF NOT EXISTS textbook_dict
(
    id          SERIAL PRIMARY KEY,
    textbook_id INTEGER REFERENCES textbook (id) ON DELETE CASCADE, -- 菜单标识
    type_code   VARCHAR(50)  NOT NULL,                              -- 如: 'question_type', 'question_tag'
    item_value  VARCHAR(100) NOT NULL,                              -- 如: '选择题'
    sort_order  INT         DEFAULT 0,                              -- 排序
    is_select   BOOLEAN     DEFAULT FALSE,                          -- 新增字段：是否选中，默认设为 false
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (textbook_id, type_code, item_value)                     -- 确保同一类型下 value 唯一
);

-- 5. 题目表
CREATE TABLE IF NOT EXISTS question
(
    id               BIGSERIAL PRIMARY KEY,

    -- 基础关联
    question_cate_id INTEGER  NOT NULL,                                                         -- 题型标识
    question_type_id INTEGER  NOT NULL,                                                         -- 题目类型ID (1:单选, 2:多选, 3:填空, 4:解答等)
    question_tag_ids JSONB,                                                                     -- 题目标签IDs
    author_id        BIGINT   NOT NULL,                                                         -- 创作者标识

    -- 核心内容
    title            TEXT     NOT NULL,                                                         -- 标题 (支持 LaTeX 字符串)
    content_plain    TEXT,                                                                      -- 增加纯文本搜索字段
    comment          TEXT,                                                                      -- 补充说明

    difficulty_level DECIMAL(2, 1) CHECK (difficulty_level >= 1.0 AND difficulty_level <= 5.0), -- 难易度

    -- 资源与附件
    images           JSONB             DEFAULT '[]'::jsonb,                                     -- 图片地址列表 ["url1", "url2"]

    -- 选项：仅选择题使用，存储为 [{ "label": "A", "content": "..." }]
    options          JSONB             DEFAULT '[]'::jsonb,
    options_layout   SMALLINT          DEFAULT 3,                                               -- 布局方案: 1: 一行(inline), 2: 两行(双列), 3: 一列(垂直)

    -- 答案与解析
    answer           TEXT,                                                                      -- 参考答案
    knowledge        TEXT,                                                                      -- 知识点标签
    analysis         JSONB             DEFAULT '{}'::jsonb,                                     -- 解题分析
    process          JSONB             DEFAULT '{}'::jsonb,                                     -- 解题过程 (详细步骤)
    remark           TEXT,                                                                      -- 备注

    status           SMALLINT NOT NULL DEFAULT 0,                                               -- 0 草稿 1 审核中 2 审核通过 3 拒绝
    approve_id       BIGINT            DEFAULT 0,                                               -- 审核人
    reject_reason    TEXT,                                                                      -- 审核拒绝后的反馈意见
    approve_at       TIMESTAMPTZ,                                                               -- 审核时间

    -- 审计字段
    created_at       TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP
);

-- 题型和题目类型联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_type ON question (question_cate_id, question_type_id);
-- 题型和审核状态联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_status ON question (question_cate_id, status);
-- 查看作者自己的题
CREATE INDEX IF NOT EXISTS idx_author_status ON question (author_id, status);

-- 6. 变式题
CREATE TABLE IF NOT EXISTS question_similar
(
    id          BIGSERIAL PRIMARY KEY,
    question_id BIGINT NOT NULL, -- 父题主键
    child_id    BIGINT NOT NULL, -- 变式题主键
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (question_id, child_id)
);

-- 7. 组卷规则
CREATE TABLE IF NOT EXISTS rule
(
    id         SERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 8. 试卷
CREATE TABLE IF NOT EXISTS test
(
    id          BIGSERIAL PRIMARY KEY,
    textbook_id INTEGER      NOT NULL, -- 教材标识
    title       VARCHAR(255) NOT NULL, -- 试卷名称
    rule_id     INTEGER      NOT NULL, -- 规则标识
    score       INT          NOT NULL, -- 试卷总分

    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 9. 试卷详情
CREATE TABLE IF NOT EXISTS test_detail
(
    id          BIGSERIAL PRIMARY KEY,
    test_id     BIGINT       NOT NULL, -- 试卷标识
    title       VARCHAR(255) NOT NULL, -- 标题名称
    description TEXT,                  -- 标题描述, 即括号中的长内容
    score       INT          NOT NULL, -- 标题总分
    sort_order  INT         DEFAULT 0, -- 标题排序
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 10. 题目详情
CREATE TABLE IF NOT EXISTS test_detail_info
(
    id          BIGSERIAL PRIMARY KEY,
    detail_id   BIGINT NOT NULL,       -- 题目信息标识
    no          INT    NOT NULL,       -- 题目序号
    question_id BIGINT NOT NULL,       -- 题目标识
    score       INT    NOT NULL,       -- 题目分数
    sort_order  INT         DEFAULT 0, -- 题目排序
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
