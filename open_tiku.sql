-- 说明： IDE 格式化如果混乱, 可以关闭格式化, 用其它工具格式化, 真扯淡
--
-- 1. 创建教材层级表
CREATE TABLE IF NOT EXISTS textbook
(
    id         SERIAL PRIMARY KEY,
    parent_id  INTEGER REFERENCES textbook (id) ON DELETE CASCADE, -- 父级标识, 查询时需要指定 path_depth 控制深度
    label      VARCHAR(255) NOT NULL,                              -- 名称对应学段科目等
    key        VARCHAR(120) NOT NULL,                              -- 名称标识
    path_depth INTEGER,                                            -- 层级深度
    sort_order INTEGER     DEFAULT 0,                              -- 排序
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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
CREATE TABLE textbook_dict
(
    id          SERIAL PRIMARY KEY,
    textbook_id INTEGER REFERENCES textbook (id) ON DELETE CASCADE, -- 菜单标识
    type_code   VARCHAR(50)  NOT NULL,                              -- 如: 'question_type', 'question_tag'
    item_value  VARCHAR(100) NOT NULL,                              -- 如: '选择题'
    sort_order  INT         DEFAULT 0,                              -- 排序
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (textbook_id, type_code, item_value)                     -- 确保同一类型下 value 唯一
);
