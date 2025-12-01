🪶 NuisLang Whitepaper v0.4

A Language Designed to Outlive Hardware.
MIT License

⸻

0. 序 · 语义文明的宣言

当计算以指令为中心，语言便服务于机器；
当计算以语义为中心，语言才服务于文明。

过去的语言以指令、结构、架构为核心，
为了追逐性能，不断向硬件妥协、向底层让渡抽象。

Nuis 拒绝这一历史惯性。

Nuis 是第五代语言：
意图优先、语义优先、调度优先。

它不为设备编程，而为意义编程；
它不是被硬件决定的语言，而是未来硬件将适配的语言。

⸻

1. Nuis 的世界观（Philosophy）

Nuis 的设计以三条稳定轴为基础：

维度	定义	属性
语义稳定性	描述“做什么”的语义层不受硬件侵蚀	永久稳定
执行可演化性	执行路径可替换，不影响语义结构	可替换
策略可学习性	编译器具备进化的策略智能	可演化

信条：
	1.	用户描述意图，执行由系统决定。
	2.	抽象建立于语义结构，而非语法糖。
	3.	时间兼容性优于平台兼容性。
	4.	编译器是策略智能体，而非规则机器。
	5.	安全来自语义一致性，而非附加规则。

⸻

2. NIR：语义意图 IR（Semantic Intent IR）

NIR 是 Nuis 的最高层语言语义。

NIR 的目标：
	•	捕获意图（Intent）
	•	捕获逻辑关系
	•	捕获资源抽象
	•	完全独立于平台、架构、执行域

在 NIR 中，程序是一组语义节点的关系图，而非指令流。

示例（概念化）：

let buf = Buffer<f32>(1024)
buf.fill(1.0)
buf.normalize()

在 NIR 中被解释为：
	•	Allocate(1024)
	•	Fill(1.0)
	•	Normalize()

NIR 不包含执行域、调度、生命周期、物理资源。

⸻

3. YIR：跨域调度 IR（Cross-Domain Scheduling IR）

YIR 是 Nuis 的核心，也是执行真理。

它承担：
	•	CPU / GPU / Shader / WASM 的跨域调度
	•	资源流（Resource Flow Graph）
	•	生命周期图（Graph Lifetime Model）
	•	域选择（Domain Selection）
	•	数据流与控制流的统一表示
	•	对接所有后端（Rust / GPU / WASM / Yalivia）

每个 YIR 程序都是一个 DAG（有向无环图）：

Node  —— 计算单元
Edge  —— 数据或控制依赖
Res   —— 资源实体（跨节点生存）

YIR 是 Nuis 的真正核心层。

⸻

4. YIR 内存模型 v0.4（Memory Model for GLM）

4.1 值分类（Value Classes）

YIR 将值分为两级：

(1) val: 短命 SSA 值
	•	算术结果、中间变量
	•	不跨节点
	•	不进入 GLM

(2) res: 资源级 Value
	•	Buffer / Image / Handle / 大对象
	•	可能跨域（CPU→GPU→CPU）
	•	需要生命周期追踪
	•	进入 GLM 分析

⸻

4.2 资源使用模式（UseMode）

每个 res 在每个 YIR Node 上有：
	•	Own
节点拥有资源，可 drop/move
	•	Read
只读，不改变所有权
	•	Write
可变借用，不改变所有权

使用模式规则：
	•	同一资源同时只能存在一个 Own（在偏序意义上）
	•	Write 与任意 Read/Write 不可并发
	•	Read 之间可以并发（前提：无 Write）

⸻

4.3 生命周期区（Region）

GLM 定义资源生命周期：

Region(R) = 所有合法使用 R 的节点集合。

构建方式：
	1.	找到资源的定义节点 N_def
	2.	收集所有 Use(res=R) 的节点
	3.	包含所有路径依赖
	4.	被 Drop/Release 节点终止

非法情况：
	•	定义前使用
	•	Drop 后使用
	•	双重所有权（Copy Own）
	•	并发 Write 冲突
	•	跨域悬挂（domain mismatch）

⸻

4.4 跨域行为

YIR 支持以下执行域：
	•	CPU
	•	GPU
	•	Shader
	•	WASM
	•	Future Domain（可扩展）

跨域移动：

send %buf -> GPU

语义：
	•	Own 迁移到 GPU
	•	Region 延伸
	•	CPU 之后不能使用 buf（除非再迁回）

跨域移动是 GLM 的核心检查点。

⸻

4.5 最小合法示例

N1: alloc  %buf (CPU)      ; Own
N2: write  %buf (CPU)
N3: send   %buf -> GPU     ; Own moves
N4: read   %buf (GPU)
N5: read   %buf (GPU)
N6: send   %buf -> CPU     ; Own moves back
N7: read   %buf (CPU)
N8: drop   %buf (CPU)

Region(%buf) = {N1..N8}

任何 N8 之后访问 buf → 编译错误。

⸻

5. Domain IR（CPU / GPU / Shader）

Domain IR 是 YIR 的物理特化层：

CPU Domain IR（CIR）
	•	类 MIR（Rust 中层 IR）
	•	显式控制流
	•	显式资源 Drop
	•	交由 Rust / LLVM 实现

GPU Domain IR（GIR）
	•	结构化 SPIR-V / compute-shader 模型
	•	显式访存、线程网格语义
	•	映射 Vulkan/Metal

Shader Domain IR（SDFG/SSA）
	•	图形着色器执行模型
	•	并行通信与材质语义

每个 Domain IR 都是 YIR 的特化。

⸻

6. Nurs：YIR-CPU ↔ Rust MIR 的双向桥接层

Nurs 的核心设计：

Nurs is a bidirectional semantic bridge between YIR-CPU and Rust MIR.

原因：
	•	YIR-CPU 特化后与 MIR 高度同构
	•	控制流可相互转换
	•	类型结构兼容
	•	生命周期在 GLM 层已保证
	•	可实现双向 lowering/lifting

6.1 Nuis → Rust

YIR-CPU
  ↓
Nurs Lowering
  ↓
Rust MIR
  ↓
LLVM
  ↓
Native Code

6.2 Rust → Nuis

Rust 模块可通过 MIR → YIR-CPU 导入：

Rust AST
  ↓
MIR
  ↓
Nurs Lifting
  ↓
YIR-CPU NodeGraph
  ↓
Nuis 程序可直接使用

无需 C ABI，不需 bindgen。

Nuis 与 Rust 在 CPU 域属于IR 兄弟关系。

6.3 稳定性模型
	•	MIR 不稳定
	•	YIR 为主轴
	•	Nurs 感知 MIR 演化
	•	Nuis 保持自身语义稳定
	•	将来 YIR 可能反向影响 MIR

⸻

7. Go 的角色：Yalivia 的一级扩展语言
	•	Go 在 Nuis 中不进入 CPU Domain
	•	Go 经由：

Go → Yalivia → YIR

参与调度
	•	动态特性（GC、JIT、反射、脚本化）完全托管 Yalivia
	•	Nuis Native 完全不受影响

Nuis Native = Rust 级高性能、零 GC
Yalivia = 动态 & 多语言世界的执行层

⸻

8. 安全模型（Semantic Safety Model）

安全由三层保障：
	1.	GLM（语义级生命周期）
	2.	Nurs（语义到 CPU 物理层的一致性）
	3.	Rust Borrow Checker（CPU 地址级安全）

三层互补，构成长久、跨域的可靠性体系。

⸻

9. Yalivia Runtime 的定位

Yalivia 是可选的运行时：
	•	动态调度（JIT / 解释）
	•	脚本绑定（Go、Python、Lua…）
	•	多语言桥接
	•	策略智能体的运行空间
	•	调度图执行引擎参考实现

原则：

Nuis Native 不依赖 Yalivia。

Yalivia 是能力扩展，不是必需品。

⸻

10. 工具链体系（Tools）

10.1 nuis-cli
	•	源码 → NIR
	•	NIR → YIR
	•	GLM 验证
	•	IR 导出（.nir, .yir, .dot, .json）
	•	调度图可视化
	•	与 nuis-rc 通信执行

示例：

nuis run main.ns
nuis ir --yir main.ns -o graph.dot
nuis schedule graph


⸻

10.2 nuis-build

构建系统：
	•	多模块工程
	•	多域构建（native/wasm/yalivia）
	•	与 Cargo/CMake 集成
	•	YIR pass pipeline 配置

⸻

10.3 nuis-rc（runtime controller）
	•	本地或远程执行 YIR
	•	CPU / GPU / WASM domain runner
	•	支持 Yalivia runtime
	•	性能分析与调度 Trace

⸻

10.4 IDE / LSP 支持
	•	基于 NIR 的语义高亮
	•	调度图可视化
	•	生命周期 Region 预览
	•	GLM 诊断
	•	nuis-rc 会话控制

⸻

11. 执行模型总览

User Code
  ↓
NIR (Semantic Intent)
  ↓
YIR (Scheduling Graph)
  ↓
GLM (Graph Lifetime Model)
  ↓
Domain IR (CPU/GPU/WASM)
  ↓
Nurs / Yalivia / WASM Backend
  ↓
Rust / GPU Driver / WASM Engine
  ↓
Hardware


⸻

12. v0.4 完成情况

组件	状态
NIR	稳定
YIR	核心结构完备
YIR 内存模型	v0.4 新增，正式落地
GLM	编译期图生命周期模型完成
Nurs	双向语义桥机制明确
工具链	完整保留并扩展
Rust 关系	定型
Go / Yalivia	定型


⸻

13. 路线图（Roadmap）

版本	内容
v0.4	GLM + YIR 内存模型 + Nurs 双向机制
v0.5	YIR Execution Prototype（可执行 DAG）
v0.6	Nurs 低保真原型（YIR-CPU → MIR）
v0.7	Yalivia Runtime 原型
v1.0	NuisLanguageOS 发布


⸻
终章 · 语言若明，文明可久

语言若只追逐性能，终将被硬件抛弃；
语言若能守住语义，便可跨越时代。

Nuis 不为平台而生，而为意义而生。

它不是语言的延续，而是语言的反思。

当计算终于理解意图，它才真正步入文明。

⸻

License：MIT
Repository：github.com/Team-silvortex/nuislang
