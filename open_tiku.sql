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

-- 1.1. 章节小节和知识点关联
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

-- 1.2. 题型列表存储
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

-- 1.3. 题型相关的字典
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

-- 2. 题目表
CREATE TABLE IF NOT EXISTS question
(
    id               BIGSERIAL PRIMARY KEY,

    -- 基础关联
    question_cate_id INTEGER      NOT NULL,                                                     -- 题型标识
    question_type_id INTEGER      NOT NULL,                                                     -- 题目类型ID (1:单选, 2:多选, 3:填空, 4:解答等)
    question_tag_ids JSONB,                                                                     -- 题目标签IDs
    author_id        BIGINT       NOT NULL,                                                     -- 创作者标识
    source           VARCHAR(500) NOT NULL DEFAULT '',                                          -- 题目来源
    original_name    VARCHAR(500) NOT NULL DEFAULT '',                                          -- 原创者昵称

    -- 核心内容
    title            TEXT         NOT NULL,                                                     -- 标题 (支持 LaTeX 字符串)
    content_plain    TEXT,                                                                      -- 增加纯文本搜索字段
    comment          TEXT,                                                                      -- 补充说明

    difficulty_level DECIMAL(2, 1) CHECK (difficulty_level >= 1.0 AND difficulty_level <= 5.0), -- 难易度

    -- 资源与附件
    images           JSONB                 DEFAULT '[]'::jsonb,                                 -- 图片地址列表 ["url1", "url2"]

    -- 选项：仅选择题使用，存储为 [{ "label": "A", "content": "..." }]
    options          JSONB                 DEFAULT '[]'::jsonb,
    options_layout   SMALLINT              DEFAULT 3,                                           -- 布局方案: 1: 一行(inline), 2: 两行(双列), 3: 一列(垂直)

    -- 答案与解析
    answer           TEXT,                                                                      -- 参考答案
    knowledge        TEXT,                                                                      -- 知识点标签
    analysis         JSONB                 DEFAULT '{}'::jsonb,                                 -- 解题分析
    process          JSONB                 DEFAULT '{}'::jsonb,                                 -- 解题过程 (详细步骤)
    steps            JSONB                 DEFAULT '[]'::jsonb,                                 -- 解题步骤
    remark           TEXT,                                                                      -- 其它备注, 不展示记录一些信息
    remark_ext       TEXT,                                                                      -- 备注

    status           SMALLINT     NOT NULL DEFAULT 0,                                           -- 0 草稿 1 审核中 2 审核通过 3 拒绝
    approve_id       BIGINT                DEFAULT 0,                                           -- 审核人
    reject_reason    TEXT,                                                                      -- 审核拒绝后的反馈意见
    approve_at       TIMESTAMPTZ,                                                               -- 审核时间

    -- 审计字段
    created_at       TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);

-- 题型和题目类型联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_type ON question (question_cate_id, question_type_id);
-- 题型和审核状态联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_status ON question (question_cate_id, status);
-- 查看作者自己的题
CREATE INDEX IF NOT EXISTS idx_author_status ON question (author_id, status);
-- 对于历史表添加以下4个字段, 对于新表无需操作
ALTER TABLE question
    ADD COLUMN IF NOT EXISTS source        VARCHAR(500) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS original_name VARCHAR(500) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS steps         JSONB DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS remark_ext    TEXT;

-- 2.1. 变式题
CREATE TABLE IF NOT EXISTS question_similar
(
    id          BIGSERIAL PRIMARY KEY,
    question_id BIGINT NOT NULL, -- 父题主键
    child_id    BIGINT NOT NULL, -- 变式题主键
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (question_id, child_id)
);

-- 3. 任务管理
CREATE TABLE IF NOT EXISTS task
(
    id               BIGSERIAL PRIMARY KEY,
    question_cate_id BIGINT       NOT NULL, -- 题型标识
    task_type        SMALLINT     NOT NULL, -- 任务类型 1 题目上传
    name             VARCHAR(128) NOT NULL, -- 文件名称
    author_id        BIGINT       NOT NULL, -- 创作者标识
    textbook_id      INTEGER      NOT NULL, -- 教材标识
    url              VARCHAR(128) NOT NULL, -- 文件路径
    email            VARCHAR(128) NOT NULL, -- 接收任务结果的邮箱
    status           SMALLINT     NOT NULL, -- 任务处理状态 1 待处理 2 处理中 3 处理成功 10 处理失败
    result           TEXT         NULL,     -- 处理结果, 成功时记录处理数量, 失败时记录错误信息, 可能不全
    created_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- 查看作者自己的任务
CREATE INDEX IF NOT EXISTS idx_cate_task ON task (question_cate_id, author_id, task_type);

-- 4. 试卷主表
CREATE TABLE paper
(
    id            BIGSERIAL PRIMARY KEY,
    related_id    INTEGER      NOT NULL,           -- 关联考点或者教材等标识, 只记录末级
    related_name  VARCHAR(255) NOT NULL,           -- 关联标识名称, 不考虑被更新仅仅是展示提醒用
    tag           VARCHAR(100) NOT NULL,           -- 标签前端写死维护
    year          VARCHAR(10)  NOT NULL,           -- 年份
    grade         VARCHAR(50)  NOT NULL,           -- 年级
    semester      VARCHAR(20)  NOT NULL,           -- 学期
    title         VARCHAR(500) NOT NULL,           -- 试卷标题
    score         INTEGER      NOT NULL DEFAULT 0, -- 总分
    source        VARCHAR(500) NOT NULL,           -- 来源, 需要记录试卷来源
    remark        TEXT,                            -- 试卷备注
    author_id     BIGINT       NOT NULL,           -- 上传者
    author_name   VARCHAR(100) NOT NULL,           -- 上传者原始昵称
    count         INTEGER      NOT NULL DEFAULT 0, -- 小题数量
    remark_ext    TEXT,                            -- 备注
    status        SMALLINT     NOT NULL DEFAULT 0, -- 0 草稿 1 审核中 2 审核通过 3 拒绝
    approve_id    BIGINT       NOT NULL DEFAULT 0, -- 审核人
    reject_reason TEXT,                            -- 审核拒绝后的反馈意见
    approve_at    TIMESTAMPTZ,                     -- 审核时间

    -- 审计字段
    created_at    TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_paper_related_id ON paper (related_id);
CREATE INDEX idx_paper_tag ON paper (tag);
CREATE INDEX idx_paper_year ON paper (year);
CREATE INDEX idx_paper_author_id ON paper (author_id);
CREATE INDEX idx_paper_status ON paper (status);

-- 4.1. 题型分组表
CREATE TABLE paper_group
(
    id        BIGINT PRIMARY KEY,    -- 业务自己生成, 不自增
    paper_id  BIGINT       NOT NULL, -- 试卷主表标识
    gen_id    VARCHAR(50)  NOT NULL, -- 前端自己生成的标识
    type_name VARCHAR(100) NOT NULL, -- 题型名称, 比如 一 选择题
    sub_title VARCHAR(500)           -- 题型说明, 比如 本题共5小题...
);
CREATE INDEX idx_paper_group_paper_id ON paper_group (paper_id);

-- 4.2 题目表
CREATE TABLE paper_question
(
    id       BIGSERIAL PRIMARY KEY,
    paper_id BIGINT      NOT NULL,                     -- 试卷主表标识
    group_id BIGINT      NOT NULL,                     -- 题型分类标识
    gen_id   VARCHAR(50) NOT NULL,                     -- 前端生成标识
    stem     TEXT        NOT NULL,                     -- 题干
    images   JSONB                DEFAULT '[]'::jsonb, -- 题目包含的图片地址标识
    options  JSONB                DEFAULT '[]'::jsonb, -- 选项内容
    answer   TEXT,                                     -- 参考答案
    analysis JSONB                DEFAULT '{}'::jsonb, -- 解题分析, 解题过程等
    score    INTEGER     NOT NULL DEFAULT 0            -- 题目分数, 不校验
);
CREATE INDEX idx_paper_question_group_id ON paper_question (paper_id, group_id);
