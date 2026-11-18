-- 说明： IDE 格式化如果混乱, 可以关闭格式化, 用其它工具格式化
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
    path       VARCHAR(500) NOT NULL DEFAULT '',                   -- 菜单路径, 记录从根节点到当前节点的父路径, 用 / 分割
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
    related_id INTEGER      NOT NULL, -- 关联标识
    label      VARCHAR(255) NOT NULL, -- 题型名称
    key        VARCHAR(120) NOT NULL, -- 题型标识
    sort_order INTEGER     DEFAULT 0, -- 排序
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
    id                     BIGSERIAL PRIMARY KEY,

    -- 基础关联
    two_level_id           INTEGER      NOT NULL DEFAULT 0,                                           -- 第2层菜单标识
    five_level_id          INTEGER      NOT NULL DEFAULT 0,                                           -- 第5层菜单标识
    question_cate_id       INTEGER      NOT NULL,                                                     -- 题型标识
    question_type_id       INTEGER      NOT NULL,                                                     -- 题目类型ID (1:单选, 2:多选, 3:填空, 4:解答等)
    question_tag_ids       JSONB,                                                                     -- 题目标签IDs
    question_dimension_ids JSONB                 DEFAULT '[]'::jsonb,                                 -- 核心素养IDs
    level_id               INTEGER      NOT NULL DEFAULT 0,                                           -- 分层体系
    scene_ids              JSONB        NOT NULL DEFAULT '[]'::jsonb,                                 -- 适用场景
    mistake_tip_ids        JSONB        NOT NULL DEFAULT '[]'::jsonb,                                 -- 常见错误
    relation_type          SMALLINT     NOT NULL DEFAULT 3,                                           -- 题目类型 1 变式题 2 课本原题 3 母题
    author_id              BIGINT       NOT NULL,                                                     -- 创作者标识
    source                 VARCHAR(500) NOT NULL DEFAULT '',                                          -- 题目来源
    original_name          VARCHAR(500) NOT NULL DEFAULT '',                                          -- 原创者昵称

    -- 核心内容
    title                  TEXT         NOT NULL,                                                     -- 标题 (支持 LaTeX 字符串)
    content_plain          TEXT,                                                                      -- 增加纯文本搜索字段
    comment                TEXT,                                                                      -- 补充说明

    difficulty_level       DECIMAL(2, 1) CHECK (difficulty_level >= 1.0 AND difficulty_level <= 5.0), -- 难易度

    -- 资源与附件
    images                 JSONB                 DEFAULT '[]'::jsonb,                                 -- 图片地址列表 ["url1", "url2"]

    -- 选项：仅选择题使用，存储为 [{ "label": "A", "content": "..." }]
    options                JSONB                 DEFAULT '[]'::jsonb,
    options_layout         SMALLINT              DEFAULT 3,                                           -- 布局方案: 1: 1列 2: 2列 3: 4列

    -- 答案与解析
    answer                 TEXT,                                                                      -- 参考答案
    knowledge              TEXT,                                                                      -- 知识点标签
    analysis               JSONB                 DEFAULT '{}'::jsonb,                                 -- 解题分析
    process                JSONB                 DEFAULT '{}'::jsonb,                                 -- 解题过程 (详细步骤)
    steps                  JSONB                 DEFAULT '[]'::jsonb,                                 -- 解题步骤
    remark                 TEXT,                                                                      -- 其它备注, 不展示记录一些信息
    remark_ext             TEXT,                                                                      -- 备注

    status                 SMALLINT     NOT NULL DEFAULT 0,                                           -- 0 草稿 1 审核中 2 审核通过 3 拒绝
    approve_id             BIGINT                DEFAULT 0,                                           -- 审核人
    reject_reason          TEXT,                                                                      -- 审核拒绝后的反馈意见
    approve_at             TIMESTAMPTZ,                                                               -- 审核时间

    -- 审计字段
    created_at             TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at             TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);

-- 题型和题目类型联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_type ON question (two_level_id, five_level_id, question_cate_id, question_type_id);
-- 题型和审核状态联合索引
CREATE INDEX IF NOT EXISTS idx_question_cate_status ON question (two_level_id, five_level_id, question_cate_id, status);
-- 查看作者自己的题
CREATE INDEX IF NOT EXISTS idx_author_status ON question (two_level_id, five_level_id, author_id, status);

-- 2.1. 题目关联关系
CREATE TABLE IF NOT EXISTS question_relation
(
    id            BIGSERIAL PRIMARY KEY,
    question_type SMALLINT NOT NULL DEFAULT 1, -- 题目类型 1 变式题, 2 课本原题
    question_id   BIGINT   NOT NULL,           -- 父题主键
    child_id      BIGINT   NOT NULL,           -- 变式题主键
    created_at    TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (question_id, child_id)
);

-- 3. 任务管理
CREATE TABLE IF NOT EXISTS task
(
    id               BIGSERIAL PRIMARY KEY,
    two_level_id     INTEGER      NOT NULL DEFAULT 0, -- 第2层菜单标识
    five_level_id    INTEGER      NOT NULL DEFAULT 0, -- 第5层菜单标识
    question_cate_id BIGINT       NOT NULL,           -- 题型标识
    task_type        SMALLINT     NOT NULL,           -- 任务类型 1 题目上传
    name             VARCHAR(128) NOT NULL,           -- 文件名称
    author_id        BIGINT       NOT NULL,           -- 创作者标识
    url              VARCHAR(128) NOT NULL,           -- 文件路径
    email            VARCHAR(128) NOT NULL,           -- 接收任务结果的邮箱
    status           SMALLINT     NOT NULL,           -- 任务处理状态 1 待处理 2 处理中 3 处理成功 10 处理失败
    result           TEXT         NULL,               -- 处理结果, 成功时记录处理数量, 失败时记录错误信息, 可能不全
    created_at       TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_cate_task ON task (author_id, two_level_id, five_level_id, question_cate_id, task_type);

-- 4. 试卷主表
CREATE TABLE paper
(
    id              BIGSERIAL PRIMARY KEY,
    two_level_id    INTEGER      NOT NULL DEFAULT 0,           -- 第2层菜单标识
    five_level_id   INTEGER      NOT NULL DEFAULT 0,           -- 第5层菜单标识
    paper_type      SMALLINT     NOT NULL DEFAULT 1,           -- 试卷类型 1 精选试卷 2 手动组卷
    file_type       SMALLINT     NOT NULL DEFAULT 0,           -- 文件类型 0 无意义 1 PDF 文件 2 Word 文件 3 图片 4 Markdown 文档
    file_ids        JSONB                 DEFAULT '[]'::jsonb, -- 文件地址信息
    province_id     SMALLINT     NOT NULL DEFAULT 0,           -- 省标识
    city_id         SMALLINT     NOT NULL DEFAULT 0,           -- 城市标识
    school_name     VARCHAR(500) NOT NULL DEFAULT '',          -- 学校名称
    tag             VARCHAR(100) NOT NULL,                     -- 标签前端写死维护
    year            VARCHAR(10)  NOT NULL,                     -- 年份
    grade           VARCHAR(50)  NOT NULL,                     -- 年级
    semester        VARCHAR(20)  NOT NULL,                     -- 学期
    title           VARCHAR(500) NOT NULL,                     -- 试卷标题
    score           INTEGER      NOT NULL DEFAULT 0,           -- 总分
    duration        SMALLINT     NOT NULL DEFAULT 0,           -- 时长, 秒
    level_type      SMALLINT     NOT NULL DEFAULT 1,           -- 试卷类型 1 普通 2 精品
    difficulty_type SMALLINT     NOT NULL DEFAULT 1,           -- 难度类型 1 容易 2 适中 3 困难
    source          VARCHAR(500) NOT NULL,                     -- 来源, 需要记录试卷来源
    remark          TEXT,                                      -- 试卷备注
    author_id       BIGINT       NOT NULL,                     -- 上传者
    author_name     VARCHAR(100) NOT NULL,                     -- 上传者原始昵称
    count           INTEGER      NOT NULL DEFAULT 0,           -- 小题数量
    remark_ext      TEXT,                                      -- 备注
    status          SMALLINT     NOT NULL DEFAULT 0,           -- 0 草稿 1 审核中 2 审核通过 3 拒绝
    approve_id      BIGINT       NOT NULL DEFAULT 0,           -- 审核人
    reject_reason   TEXT,                                      -- 审核拒绝后的反馈意见
    approve_at      TIMESTAMPTZ,                               -- 审核时间

    -- 审计字段
    created_at      TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_paper_related ON paper (two_level_id, five_level_id);
CREATE INDEX idx_author_paper_related ON paper (author_id, two_level_id, five_level_id);

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
    id             BIGSERIAL PRIMARY KEY,
    paper_id       BIGINT      NOT NULL,                     -- 试卷主表标识
    group_id       BIGINT      NOT NULL,                     -- 题型分类标识
    gen_id         VARCHAR(50) NOT NULL,                     -- 前端生成标识
    order_num      SMALLINT    NOT NULL DEFAULT 1,           -- 题目顺序编号
    stem           TEXT        NOT NULL,                     -- 题干
    images         JSONB                DEFAULT '[]'::jsonb, -- 题目包含的图片地址标识
    options        JSONB                DEFAULT '[]'::jsonb, -- 选项内容
    options_layout SMALLINT             DEFAULT 3,           -- 布局方案: 1: 1列 2: 2列 3: 4列
    answer         TEXT,                                     -- 参考答案
    analysis       JSONB                DEFAULT '{}'::jsonb, -- 解题分析, 解题过程等
    score          INTEGER     NOT NULL DEFAULT 0            -- 题目分数, 不校验
);
CREATE INDEX idx_paper_question_group_id ON paper_question (paper_id, group_id);

-- 4.3 组卷题目生成规则
CREATE TABLE paper_gen_config
(
    id                     BIGSERIAL PRIMARY KEY,
    paper_id               BIGINT NOT NULL,                     -- 试卷主表标识
    question_cate_ids      JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 题型标识IDs
    question_tag_ids       JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 题目标签IDs
    question_dimension_ids JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 核心素养IDs
    question_type_info     JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 题型题量信息
    difficulty_level_info  JSONB  NOT NULL DEFAULT '{}'::jsonb, -- 难度分布信息
    level_ids              JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 分层体系
    scene_ids              JSONB  NOT NULL DEFAULT '[]'::jsonb, -- 适用场景
    mistake_tip_ids        JSONB  NOT NULL DEFAULT '[]'::jsonb  -- 常见错误
);
CREATE INDEX idx_paper_gen_config_paper_id ON paper_gen_config (paper_id);

-- 4.4 试卷生成的题目信息
CREATE TABLE paper_gen_question
(
    id          BIGSERIAL PRIMARY KEY,
    paper_id    BIGINT      NOT NULL,           -- 试卷主表标识
    group_id    BIGINT      NOT NULL,           -- 题型分类标识
    gen_id      VARCHAR(50) NOT NULL,           -- 前端生成标识
    order_num   SMALLINT    NOT NULL DEFAULT 1, -- 题目顺序编号
    question_id BIGINT      NOT NULL,           -- 题目标识
    score       INTEGER     NOT NULL DEFAULT 0  -- 题目分数, 不校验
);
CREATE INDEX idx_paper_gen_question_group_id ON paper_gen_question (paper_id, group_id);

-- 5. 第三方身份认证表（存储所有OAuth登录源）
CREATE TABLE user_identity
(
    id                BIGSERIAL PRIMARY KEY,
    user_id           BIGINT       NOT NULL,

    -- 三方平台标识
    provider          SMALLINT     NOT NULL,            -- 1 github, 2 qq
    provider_user_id  VARCHAR(100) NOT NULL,            -- 第三方平台返回的 OpenID / UserID

    -- 第三方返回的公开信息
    provider_username VARCHAR(100),                     -- 三方平台昵称
    provider_email    VARCHAR(255),                     -- 三方平台邮箱

    -- 登录
    last_login_time   TIMESTAMPTZ,                      -- 最后一次登录时间
    login_count       BIGINT                DEFAULT 0,

    role              SMALLINT     NOT NULL,            -- 用户角色 1 普通 2 学生 3 教师
    status            SMALLINT     NOT NULL,            -- 用户状态 1 正常 2 暂停 20 封禁

    remark            TEXT         NOT NULL DEFAULT '', -- 备注

    created_at        TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,

    -- 唯一约束：同一平台下的同一个用户ID只能绑定一次
    CONSTRAINT unique_provider_user UNIQUE (provider, provider_user_id)
);
CREATE INDEX idx_user_id ON user_identity (user_id);

-- 5.1 用户登录状态, 目前用户比较少, 登录 session 直接记录在数据库中
CREATE TABLE user_session
(
    id         BIGSERIAL PRIMARY KEY,
    user_id    BIGINT       NOT NULL,
    source     SMALLINT     NOT NULL DEFAULT 1,  -- 用户来源 1 普通三方用户 2 学生账户
    token      VARCHAR(100) NOT NULL,            -- 登录 token
    expired_at TIMESTAMPTZ,                      -- 过期时间止, 过期后定时任务直接删除该条数据
    renew_cnt  SMALLINT              DEFAULT 0,  -- 续期次数
    client_ip  VARCHAR(100) NOT NULL DEFAULT '', -- 客户端ip
    user_agent VARCHAR(500) NOT NULL DEFAULT '', -- 用户请求头信息
    created_at TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_token ON user_session (token);

-- 6. 班级
CREATE TABLE class
(
    id         BIGSERIAL PRIMARY KEY,
    year       VARCHAR(10)  NOT NULL,            -- 年份
    grade      VARCHAR(50)  NOT NULL DEFAULT '', -- 年级
    semester   VARCHAR(20)  NOT NULL DEFAULT '', -- 学期
    label      VARCHAR(500) NOT NULL,            -- 班级名称
    sort_order SMALLINT     NOT NULL DEFAULT 0,  -- 排序
    email      VARCHAR(128) NOT NULL DEFAULT '', -- 接收结果的邮箱
    author_id  BIGINT       NOT NULL,            -- 作者
    remark     TEXT                  DEFAULT '', -- 备注
    created_at TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_author_year ON class (author_id, year);

-- 6.1 班级学生账户
CREATE TABLE class_student
(
    id              BIGSERIAL PRIMARY KEY,
    class_id        BIGINT       NOT NULL,            -- 班级标识
    user_id         BIGINT       NOT NULL,            -- 学生账户标识
    account         VARCHAR(128) NOT NULL,            -- 账户名称
    password        VARCHAR(128) NOT NULL,            -- 密码
    status          SMALLINT     NOT NULL DEFAULT 1,  -- 账户状态, 1 正常 2 暂停 3 停用
    remark          VARCHAR(250) NOT NULL DEFAULT '', -- 备注说明
    -- 登录
    last_login_time TIMESTAMPTZ,                      -- 最后一次登录时间
    login_count     BIGINT                DEFAULT 0,

    created_at      TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ           DEFAULT CURRENT_TIMESTAMP,

    -- 账户名称唯一
    CONSTRAINT unique_account UNIQUE (account)
);
CREATE INDEX idx_class_id ON class_student (class_id);
CREATE INDEX idx_user_id ON class_student (user_id);
CREATE INDEX idx_account ON class_student (account);

-- 7 班级作业
CREATE TABLE homework
(
    id          BIGSERIAL PRIMARY KEY,
    batch_no    INTEGER        NOT NULL,                           -- 批次号
    homework_id BIGINT         NOT NULL,                           -- 作业标识, 一次多个班级
    paper_id    BIGINT         NOT NULL,                           -- 试卷标识
    class_id    BIGINT         NOT NULL,                           -- 班级标识
    author_id   BIGINT         NOT NULL,                           -- 作者标识
    title       CHARACTER(500) NOT NULL DEFAULT '',                -- 作业名称
    deadline    TIMESTAMPTZ    NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 作业截止时间
    remark      TEXT           NOT NULL DEFAULT '',                -- 备注

    -- 审计字段
    created_at  TIMESTAMPTZ             DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ             DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_paper_batch_no ON homework (paper_id, batch_no);
CREATE INDEX idx_paper_author_id ON homework (paper_id, author_id);

-- 7.1 班级作业学生列表
CREATE TABLE homework_student
(
    id          BIGSERIAL PRIMARY KEY,
    homework_id BIGINT NOT NULL, -- 班级作业主表标识
    student_id  BIGINT NOT NULL, -- 学生账户标识

    -- 审计字段
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_homework_id ON homework_student (homework_id);
CREATE INDEX idx_homework_student_created
    ON homework_student (student_id, created_at);

-- 7.2 学生做题记录表
CREATE TABLE test_attempt
(
    id             BIGSERIAL PRIMARY KEY,
    student_id     BIGINT   NOT NULL,                           -- 学生 ID
    homework_id    BIGINT   NOT NULL,                           -- 作业标识
    class_id       BIGINT   NOT NULL,                           -- 班级标识
    paper_id       BIGINT   NOT NULL,                           -- 试卷 ID
    attempt_number SMALLINT NOT NULL DEFAULT 1,                 -- 刷题轮次/批次 第1次刷 第2次刷...
    method         SMALLINT NOT NULL DEFAULT 1,                 -- 训练方法 1 练习模式 2 考试模式
    status         SMALLINT NOT NULL DEFAULT 1,                 -- 状态：1 进行中 2 已交卷
    score          NUMERIC(5, 1)     DEFAULT 0.0,               -- 最终总得分 交卷前为0
    created_at     TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP, -- 开始时间
    updated_at     TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP, -- 进度更新时间, 减去开始时间为耗时
    completed_at   TIMESTAMPTZ,                                 -- 交卷时间

    CONSTRAINT unique_homework_student_attempt UNIQUE (homework_id, student_id, attempt_number)
);
CREATE INDEX idx_homework_student_statue ON test_attempt (homework_id, student_id, status);

-- 7.3 学生做题明细记录表
CREATE TABLE test_answer
(
    id          BIGSERIAL PRIMARY KEY,
    attempt_id  BIGINT   NOT NULL,            -- 做题记录标识
    question_id BIGINT   NOT NULL,            -- 原始题目 ID
    answer      TEXT     NOT NULL DEFAULT '', -- 用户的最终选择/填写内容
    result      SMALLINT NOT NULL DEFAULT 0,  -- 是否正确 0 未作答 1 正确 2 错误
    note        TEXT     NOT NULL DEFAULT '', -- 笔记
    remark      TEXT     NOT NULL DEFAULT '', -- 备注
    created_at  TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP,

    -- 保证在同一次刷题/考试中 一道题有且仅有一行记录
    CONSTRAINT unique_attempt_question UNIQUE (attempt_id, question_id)
);

-- 8 统计面板, 均为统计上一天
CREATE TABLE stat
(
    date                DATE PRIMARY KEY,                   -- 统计该日期的数据, 部分是昨日概念, 部分是最近概念
    latest_question_ids JSONB NOT NULL DEFAULT '[]'::jsonb, -- 最新题目标识, 最近概念
    top_question_ids    JSONB NOT NULL DEFAULT '[]'::jsonb, -- 热门题目标识
    top_textbook_info   JSONB NOT NULL DEFAULT '[]'::jsonb, -- 热门教材
    top_teacher_info    JSONB NOT NULL DEFAULT '[]'::jsonb, -- 热门教师, 包括普通用户
    top_student_info    JSONB NOT NULL DEFAULT '[]'::jsonb, -- 热门学生
    count_info          JSONB NOT NULL DEFAULT '{}'::jsonb  -- 汇总统计
);

--9 个性化
CREATE TABLE custom
(
    id          BIGSERIAL PRIMARY KEY,
    author_id   BIGINT   NOT NULL DEFAULT 0,           -- 作者标识
    custom_type SMALLINT NOT NULL,                     -- 个性化类型 1 教材菜单
    textbook    JSONB    NOT NULL DEFAULT '{}'::jsonb, -- 教材层级菜单默认值
    created_at  TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ       DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_author ON custom (author_id);

-- 10 城市表
CREATE TABLE city
(
    id             INTEGER PRIMARY KEY,  -- 行政区划代码，如 '11' 或 '1101'
    parent_id      INTEGER DEFAULT NULL, -- 父级ID（省级为 NULL，市级指向所属省 ID）
    name           VARCHAR(64) NOT NULL, -- 省/市名称
    pinyin_initial CHAR(1)     NOT NULL, -- 汉语拼音首字母（小写）
    level          SMALLINT    NOT NULL  -- 层级：1-省/直辖市/自治区/特别行政区，2-城市
);
CREATE INDEX idx_city_parent_id ON city (parent_id);
CREATE INDEX idx_city_level_initial ON city (level, pinyin_initial);

-- 插入省级数据共 34 条
INSERT INTO city (id, parent_id, name, pinyin_initial, level)
VALUES ('11', NULL, '北京', 'b', 1),
       ('12', NULL, '天津', 't', 1),
       ('13', NULL, '河北', 'h', 1),
       ('14', NULL, '山西', 's', 1),
       ('15', NULL, '内蒙古', 'n', 1),
       ('21', NULL, '辽宁', 'l', 1),
       ('22', NULL, '吉林', 'j', 1),
       ('23', NULL, '黑龙江', 'h', 1),
       ('31', NULL, '上海', 's', 1),
       ('32', NULL, '江苏', 'j', 1),
       ('33', NULL, '浙江', 'z', 1),
       ('34', NULL, '安徽', 'a', 1),
       ('35', NULL, '福建', 'f', 1),
       ('36', NULL, '江西', 'j', 1),
       ('37', NULL, '山东', 's', 1),
       ('41', NULL, '河南', 'h', 1),
       ('42', NULL, '湖北', 'h', 1),
       ('43', NULL, '湖南', 'h', 1),
       ('44', NULL, '广东', 'g', 1),
       ('45', NULL, '广西', 'g', 1),
       ('46', NULL, '海南', 'h', 1),
       ('50', NULL, '重庆', 'c', 1),
       ('51', NULL, '四川', 's', 1),
       ('52', NULL, '贵州', 'g', 1),
       ('53', NULL, '云南', 'y', 1),
       ('54', NULL, '西藏', 'x', 1),
       ('61', NULL, '陕西', 's', 1),
       ('62', NULL, '甘肃', 'g', 1),
       ('63', NULL, '青海', 'q', 1),
       ('64', NULL, '宁夏', 'n', 1),
       ('65', NULL, '新疆', 'x', 1),
       ('71', NULL, '台湾', 't', 1),
       ('81', NULL, '香港', 'x', 1),
       ('82', NULL, '澳门', 'a', 1);

-- 插入市级数据共 365 条
INSERT INTO city (id, parent_id, name, pinyin_initial, level)
VALUES ('1101', '11', '北京', 'b', 2),
       ('1201', '12', '天津', 't', 2),
       ('1301', '13', '石家庄', 's', 2),
       ('1302', '13', '唐山', 't', 2),
       ('1303', '13', '秦皇岛', 'q', 2),
       ('1304', '13', '邯郸', 'h', 2),
       ('1305', '13', '邢台', 'x', 2),
       ('1306', '13', '保定', 'b', 2),
       ('1307', '13', '张家口', 'z', 2),
       ('1308', '13', '承德', 'c', 2),
       ('1309', '13', '沧州', 'c', 2),
       ('1310', '13', '廊坊', 'l', 2),
       ('1311', '13', '衡水', 'h', 2),
       ('1401', '14', '太原', 't', 2),
       ('1402', '14', '大同', 'd', 2),
       ('1403', '14', '阳泉', 'y', 2),
       ('1404', '14', '长治', 'c', 2),
       ('1405', '14', '晋城', 'j', 2),
       ('1406', '14', '朔州', 's', 2),
       ('1407', '14', '晋中', 'j', 2),
       ('1408', '14', '运城', 'y', 2),
       ('1409', '14', '忻州', 'x', 2),
       ('1410', '14', '临汾', 'l', 2),
       ('1411', '14', '吕梁', 'l', 2),
       ('1501', '15', '呼和浩特', 'h', 2),
       ('1502', '15', '包头', 'b', 2),
       ('1503', '15', '乌海', 'w', 2),
       ('1504', '15', '赤峰', 'c', 2),
       ('1505', '15', '通辽', 't', 2),
       ('1506', '15', '鄂尔多斯', 'e', 2),
       ('1507', '15', '呼伦贝尔', 'h', 2),
       ('1508', '15', '巴彦淖尔', 'b', 2),
       ('1509', '15', '乌兰察布', 'w', 2),
       ('1522', '15', '兴安', 'x', 2),
       ('1525', '15', '锡林郭勒', 'x', 2),
       ('1529', '15', '阿拉善', 'a', 2),
       ('2101', '21', '沈阳', 's', 2),
       ('2102', '21', '大连', 'd', 2),
       ('2103', '21', '鞍山', 'a', 2),
       ('2104', '21', '抚顺', 'f', 2),
       ('2105', '21', '本溪', 'b', 2),
       ('2106', '21', '丹东', 'd', 2),
       ('2107', '21', '锦州', 'j', 2),
       ('2108', '21', '营口', 'y', 2),
       ('2109', '21', '阜新', 'f', 2),
       ('2110', '21', '辽阳', 'l', 2),
       ('2111', '21', '盘锦', 'p', 2),
       ('2112', '21', '铁岭', 't', 2),
       ('2113', '21', '朝阳', 'c', 2),
       ('2114', '21', '葫芦岛', 'h', 2),
       ('2201', '22', '长春', 'c', 2),
       ('2202', '22', '吉林市', 'j', 2),
       ('2203', '22', '四平', 's', 2),
       ('2204', '22', '辽源', 'l', 2),
       ('2205', '22', '通化', 't', 2),
       ('2206', '22', '白山', 'b', 2),
       ('2207', '22', '松原', 's', 2),
       ('2208', '22', '白城', 'b', 2),
       ('2224', '22', '延边', 'y', 2),
       ('2301', '23', '哈尔滨', 'h', 2),
       ('2302', '23', '齐齐哈尔', 'q', 2),
       ('2303', '23', '鸡西', 'j', 2),
       ('2304', '23', '鹤岗', 'h', 2),
       ('2305', '23', '双鸭山', 's', 2),
       ('2306', '23', '大庆', 'd', 2),
       ('2307', '23', '伊春', 'y', 2),
       ('2308', '23', '佳木斯', 'j', 2),
       ('2309', '23', '七台河', 'q', 2),
       ('2310', '23', '牡丹江', 'm', 2),
       ('2311', '23', '黑河', 'h', 2),
       ('2312', '23', '绥化', 's', 2),
       ('2327', '23', '大兴安岭', 'd', 2),
       ('3101', '31', '上海', 's', 2),
       ('3201', '32', '南京', 'n', 2),
       ('3202', '32', '无锡', 'w', 2),
       ('3203', '32', '徐州', 'x', 2),
       ('3204', '32', '常州', 'c', 2),
       ('3205', '32', '苏州', 's', 2),
       ('3206', '32', '南通', 'n', 2),
       ('3207', '32', '连云港', 'l', 2),
       ('3208', '32', '淮安', 'h', 2),
       ('3209', '32', '盐城', 'y', 2),
       ('3210', '32', '扬州', 'y', 2),
       ('3211', '32', '镇江', 'z', 2),
       ('3212', '32', '泰州', 't', 2),
       ('3213', '32', '宿迁', 's', 2),
       ('3301', '33', '杭州', 'h', 2),
       ('3302', '33', '宁波', 'n', 2),
       ('3303', '33', '温州', 'w', 2),
       ('3304', '33', '嘉兴', 'j', 2),
       ('3305', '33', '湖州', 'h', 2),
       ('3306', '33', '绍兴', 's', 2),
       ('3307', '33', '金华', 'j', 2),
       ('3308', '33', '衢州', 'q', 2),
       ('3309', '33', '舟山', 'z', 2),
       ('3310', '33', '台州', 't', 2),
       ('3311', '33', '丽水', 'l', 2),
       ('3401', '34', '合肥', 'h', 2),
       ('3402', '34', '芜湖', 'w', 2),
       ('3403', '34', '蚌埠', 'b', 2),
       ('3404', '34', '淮南', 'h', 2),
       ('3405', '34', '马鞍山', 'm', 2),
       ('3406', '34', '淮北', 'h', 2),
       ('3407', '34', '铜陵', 't', 2),
       ('3408', '34', '安庆', 'a', 2),
       ('3410', '34', '黄山', 'h', 2),
       ('3411', '34', '滁州', 'c', 2),
       ('3412', '34', '阜阳', 'f', 2),
       ('3413', '34', '宿州', 's', 2),
       ('3415', '34', '六安', 'l', 2),
       ('3416', '34', '亳州', 'b', 2),
       ('3417', '34', '池州', 'c', 2),
       ('3418', '34', '宣城', 'x', 2),
       ('3501', '35', '福州', 'f', 2),
       ('3502', '35', '厦门', 'x', 2),
       ('3503', '35', '莆田', 'p', 2),
       ('3504', '35', '三明', 's', 2),
       ('3505', '35', '泉州', 'q', 2),
       ('3506', '35', '漳州', 'z', 2),
       ('3507', '35', '南平', 'n', 2),
       ('3508', '35', '龙岩', 'l', 2),
       ('3509', '35', '宁德', 'n', 2),
       ('3601', '36', '南昌', 'n', 2),
       ('3602', '36', '景德镇', 'j', 2),
       ('3603', '36', '萍乡', 'p', 2),
       ('3604', '36', '九江', 'j', 2),
       ('3605', '36', '新余', 'x', 2),
       ('3606', '36', '鹰潭', 'y', 2),
       ('3607', '36', '赣州', 'g', 2),
       ('3608', '36', '吉安', 'j', 2),
       ('3609', '36', '宜春', 'y', 2),
       ('3610', '36', '抚州', 'f', 2),
       ('3611', '36', '上饶', 's', 2),
       ('3701', '37', '济南', 'j', 2),
       ('3702', '37', '青岛', 'q', 2),
       ('3703', '37', '淄博', 'z', 2),
       ('3704', '37', '枣庄', 'z', 2),
       ('3705', '37', '东营', 'd', 2),
       ('3706', '37', '烟台', 'y', 2),
       ('3707', '37', '潍坊', 'w', 2),
       ('3708', '37', '济宁', 'j', 2),
       ('3709', '37', '泰安', 't', 2),
       ('3710', '37', '威海', 'w', 2),
       ('3711', '37', '日照', 'r', 2),
       ('3713', '37', '临沂', 'l', 2),
       ('3714', '37', '德州', 'd', 2),
       ('3715', '37', '聊城', 'l', 2),
       ('3716', '37', '滨州', 'b', 2),
       ('3717', '37', '菏泽', 'h', 2),
       ('4101', '41', '郑州', 'z', 2),
       ('4102', '41', '开封', 'k', 2),
       ('4103', '41', '洛阳', 'l', 2),
       ('4104', '41', '平顶山', 'p', 2),
       ('4105', '41', '安阳', 'a', 2),
       ('4106', '41', '鹤壁', 'h', 2),
       ('4107', '41', '新乡', 'x', 2),
       ('4108', '41', '焦作', 'j', 2),
       ('4109', '41', '濮阳', 'p', 2),
       ('4110', '41', '许昌', 'x', 2),
       ('4111', '41', '漯河', 'l', 2),
       ('4112', '41', '三门峡', 's', 2),
       ('4113', '41', '南阳', 'n', 2),
       ('4114', '41', '商丘', 's', 2),
       ('4115', '41', '信阳', 'x', 2),
       ('4116', '41', '周口', 'z', 2),
       ('4117', '41', '驻马店', 'z', 2),
       ('419001', '41', '济源', 'j', 2),
       ('4201', '42', '武汉', 'w', 2),
       ('4202', '42', '黄石', 'h', 2),
       ('4203', '42', '十堰', 's', 2),
       ('4205', '42', '宜昌', 'y', 2),
       ('4206', '42', '襄阳', 'x', 2),
       ('4207', '42', '鄂州', 'e', 2),
       ('4208', '42', '荆门', 'j', 2),
       ('4209', '42', '孝感', 'x', 2),
       ('4210', '42', '荆州', 'j', 2),
       ('4211', '42', '黄冈', 'h', 2),
       ('4212', '42', '咸宁', 'x', 2),
       ('4213', '42', '随州', 's', 2),
       ('4228', '42', '恩施', 'e', 2),
       ('429004', '42', '仙桃', 'x', 2),
       ('429005', '42', '潜江', 'q', 2),
       ('429006', '42', '天门', 't', 2),
       ('429021', '42', '神农架', 's', 2),
       ('4301', '43', '长沙', 'c', 2),
       ('4302', '43', '株洲', 'z', 2),
       ('4303', '43', '湘潭', 'x', 2),
       ('4304', '43', '衡阳', 'h', 2),
       ('4305', '43', '邵阳', 's', 2),
       ('4306', '43', '岳阳', 'y', 2),
       ('4307', '43', '常德', 'c', 2),
       ('4308', '43', '张家界', 'z', 2),
       ('4309', '43', '益阳', 'y', 2),
       ('4310', '43', '郴州', 'c', 2),
       ('4311', '43', '永州', 'y', 2),
       ('4312', '43', '怀化', 'h', 2),
       ('4313', '43', '娄底', 'l', 2),
       ('4331', '43', '湘西', 'x', 2),
       ('4401', '44', '广州', 'g', 2),
       ('4402', '44', '韶关', 's', 2),
       ('4403', '44', '深圳', 's', 2),
       ('4404', '44', '珠海', 'z', 2),
       ('4405', '44', '汕头', 's', 2),
       ('4406', '44', '佛山', 'f', 2),
       ('4407', '44', '江门', 'j', 2),
       ('4408', '44', '湛江', 'z', 2),
       ('4409', '44', '茂名', 'm', 2),
       ('4412', '44', '肇庆', 'z', 2),
       ('4413', '44', '惠州', 'h', 2),
       ('4414', '44', '梅州', 'm', 2),
       ('4415', '44', '汕尾', 's', 2),
       ('4416', '44', '河源', 'h', 2),
       ('4417', '44', '阳江', 'y', 2),
       ('4418', '44', '清远', 'q', 2),
       ('4419', '44', '东莞', 'd', 2),
       ('4420', '44', '中山', 'z', 2),
       ('4451', '44', '潮州', 'c', 2),
       ('4452', '44', '揭阳', 'j', 2),
       ('4453', '44', '云浮', 'y', 2),
       ('4501', '45', '南宁', 'n', 2),
       ('4502', '45', '柳州', 'l', 2),
       ('4503', '45', '桂林', 'g', 2),
       ('4504', '45', '梧州', 'w', 2),
       ('4505', '45', '北海', 'b', 2),
       ('4506', '45', '防城港', 'f', 2),
       ('4507', '45', '钦州', 'q', 2),
       ('4508', '45', '贵港', 'g', 2),
       ('4509', '45', '玉林', 'y', 2),
       ('4510', '45', '百色', 'b', 2),
       ('4511', '45', '贺州', 'h', 2),
       ('4512', '45', '河池', 'h', 2),
       ('4513', '45', '来宾', 'l', 2),
       ('4514', '45', '崇左', 'c', 2),
       ('4601', '46', '海口', 'h', 2),
       ('4602', '46', '三亚', 's', 2),
       ('4603', '46', '三沙', 's', 2),
       ('4604', '46', '儋州', 'd', 2),
       ('469001', '46', '五指山', 'w', 2),
       ('469002', '46', '琼海', 'q', 2),
       ('469005', '46', '文昌', 'w', 2),
       ('469006', '46', '万宁', 'w', 2),
       ('469007', '46', '东方', 'd', 2),
       ('469021', '46', '定安', 'd', 2),
       ('469022', '46', '屯昌', 't', 2),
       ('469023', '46', '澄迈', 'c', 2),
       ('469024', '46', '临高', 'l', 2),
       ('469025', '46', '白沙', 'b', 2),
       ('469026', '46', '昌江', 'c', 2),
       ('469027', '46', '乐东', 'l', 2),
       ('469028', '46', '陵水', 'l', 2),
       ('469029', '46', '保亭', 'b', 2),
       ('469030', '46', '琼中', 'q', 2),
       ('5001', '50', '重庆城区', 'c', 2),
       ('5002', '50', '重庆郊县', 'c', 2),
       ('5101', '51', '成都', 'c', 2),
       ('5103', '51', '自贡', 'z', 2),
       ('5104', '51', '攀枝花', 'p', 2),
       ('5105', '51', '泸州', 'l', 2),
       ('5106', '51', '德阳', 'd', 2),
       ('5107', '51', '绵阳', 'm', 2),
       ('5108', '51', '广元', 'g', 2),
       ('5109', '51', '遂宁', 's', 2),
       ('5110', '51', '内江', 'n', 2),
       ('5111', '51', '乐山', 'l', 2),
       ('5113', '51', '南充', 'n', 2),
       ('5114', '51', '眉山', 'm', 2),
       ('5115', '51', '宜宾', 'y', 2),
       ('5116', '51', '广安', 'g', 2),
       ('5117', '51', '达州', 'd', 2),
       ('5118', '51', '雅安', 'y', 2),
       ('5119', '51', '巴中', 'b', 2),
       ('5120', '51', '资阳', 'z', 2),
       ('5132', '51', '阿坝', 'a', 2),
       ('5133', '51', '甘孜', 'g', 2),
       ('5134', '51', '凉山', 'l', 2),
       ('5201', '52', '贵阳', 'g', 2),
       ('5202', '52', '六盘水', 'l', 2),
       ('5203', '52', '遵义', 'z', 2),
       ('5204', '52', '安顺', 'a', 2),
       ('5205', '52', '毕节', 'b', 2),
       ('5206', '52', '铜仁', 't', 2),
       ('5223', '52', '黔西南', 'q', 2),
       ('5226', '52', '黔东南', 'q', 2),
       ('5227', '52', '黔南', 'q', 2),
       ('5301', '53', '昆明', 'k', 2),
       ('5303', '53', '曲靖', 'q', 2),
       ('5304', '53', '玉溪', 'y', 2),
       ('5305', '53', '保山', 'b', 2),
       ('5306', '53', '昭通', 'z', 2),
       ('5307', '53', '丽江', 'l', 2),
       ('5308', '53', '普洱', 'p', 2),
       ('5309', '53', '临沧', 'l', 2),
       ('5323', '53', '楚雄', 'c', 2),
       ('5325', '53', '红河', 'h', 2),
       ('5326', '53', '文山', 'w', 2),
       ('5328', '53', '西双版纳', 'x', 2),
       ('5329', '53', '大理', 'd', 2),
       ('5331', '53', '德宏', 'd', 2),
       ('5333', '53', '怒江', 'n', 2),
       ('5334', '53', '迪庆', 'd', 2),
       ('5401', '54', '拉萨', 'l', 2),
       ('5402', '54', '日喀则', 'r', 2),
       ('5403', '54', '昌都', 'c', 2),
       ('5404', '54', '林芝', 'l', 2),
       ('5405', '54', '山南', 's', 2),
       ('5406', '54', '那曲', 'n', 2),
       ('5425', '54', '阿里', 'a', 2),
       ('6101', '61', '西安', 'x', 2),
       ('6102', '61', '铜川', 't', 2),
       ('6103', '61', '宝鸡', 'b', 2),
       ('6104', '61', '咸阳', 'x', 2),
       ('6105', '61', '渭南', 'w', 2),
       ('6106', '61', '延安', 'y', 2),
       ('6107', '61', '汉中', 'h', 2),
       ('6108', '61', '榆林', 'y', 2),
       ('6109', '61', '安康', 'a', 2),
       ('6110', '61', '商洛', 's', 2),
       ('6201', '62', '兰州', 'l', 2),
       ('6202', '62', '嘉峪关', 'j', 2),
       ('6203', '62', '金昌', 'j', 2),
       ('6204', '62', '白银', 'b', 2),
       ('6205', '62', '天水', 't', 2),
       ('6206', '62', '武威', 'w', 2),
       ('6207', '62', '张掖', 'z', 2),
       ('6208', '62', '平凉', 'p', 2),
       ('6209', '62', '酒泉', 'j', 2),
       ('6210', '62', '庆阳', 'q', 2),
       ('6211', '62', '定西', 'd', 2),
       ('6212', '62', '陇南', 'l', 2),
       ('6229', '62', '临夏', 'l', 2),
       ('6230', '62', '甘南', 'g', 2),
       ('6301', '63', '西宁', 'x', 2),
       ('6302', '63', '海东', 'h', 2),
       ('6322', '63', '海北', 'h', 2),
       ('6323', '63', '黄南', 'h', 2),
       ('6325', '63', '海南', 'h', 2),
       ('6326', '63', '果洛', 'g', 2),
       ('6327', '63', '玉树', 'y', 2),
       ('6328', '63', '海西', 'h', 2),
       ('6401', '64', '银川', 'y', 2),
       ('6402', '64', '石嘴山', 's', 2),
       ('6403', '64', '吴忠', 'w', 2),
       ('6404', '64', '固原', 'g', 2),
       ('6405', '64', '中卫', 'z', 2),
       ('6501', '65', '乌鲁木齐', 'w', 2),
       ('6502', '65', '克拉玛依', 'k', 2),
       ('6504', '65', '吐鲁番', 't', 2),
       ('6505', '65', '哈密', 'h', 2),
       ('6523', '65', '昌吉', 'c', 2),
       ('6527', '65', '博尔塔拉', 'b', 2),
       ('6528', '65', '巴音郭楞', 'b', 2),
       ('6529', '65', '阿克苏', 'a', 2),
       ('6530', '65', '克孜勒苏', 'k', 2),
       ('6531', '65', '喀什', 'k', 2),
       ('6532', '65', '和田', 'h', 2),
       ('6540', '65', '伊犁', 'y', 2),
       ('6542', '65', '塔城', 't', 2),
       ('6543', '65', '阿勒泰', 'a', 2),
       ('659001', '65', '石河子', 's', 2),
       ('659002', '65', '阿拉尔', 'a', 2),
       ('659003', '65', '图木舒克', 't', 2),
       ('659004', '65', '五家渠', 'w', 2),
       ('659005', '65', '北屯', 'b', 2),
       ('659006', '65', '铁门关', 't', 2),
       ('659007', '65', '双河', 's', 2),
       ('659008', '65', '可克达拉', 'k', 2),
       ('659009', '65', '昆玉', 'k', 2),
       ('659010', '65', '胡杨河', 'h', 2),
       ('659011', '65', '新星', 'x', 2),
       ('659012', '65', '白杨', 'b', 2),
       ('7101', '71', '台北', 't', 2),
       ('7102', '71', '高雄', 'g', 2),
       ('7103', '71', '台南', 't', 2),
       ('7104', '71', '台中', 't', 2),
       ('7106', '71', '南投', 'n', 2),
       ('7107', '71', '基隆', 'j', 2),
       ('7108', '71', '新竹市', 'x', 2),
       ('7109', '71', '嘉义市', 'j', 2),
       ('7111', '71', '新北', 'x', 2),
       ('7112', '71', '宜兰', 'y', 2),
       ('7113', '71', '新竹县', 'x', 2),
       ('7114', '71', '桃园', 't', 2),
       ('7115', '71', '苗栗', 'm', 2),
       ('7117', '71', '彰化', 'z', 2),
       ('7119', '71', '嘉义县', 'j', 2),
       ('7121', '71', '云林', 'y', 2),
       ('7124', '71', '屏东', 'p', 2),
       ('7125', '71', '台东', 't', 2),
       ('7126', '71', '花莲', 'h', 2),
       ('7127', '71', '澎湖', 'p', 2),
       ('8100', '81', '香港', 'x', 2),
       ('8200', '82', '澳门', 'a', 2);
