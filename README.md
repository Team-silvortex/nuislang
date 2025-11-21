🪶 NuisLang Whitepaper v0.2

A Language Designed to Outlive Hardware
语言的使命不是追随硬件，而是超越它。

⸻

序 · 语义文明的宣言

当计算诞生于指令之时，它注定服务于机器；
当语言诞生于语义之上，它才开始服务于文明。

过去几十年，我们将语言视作指令的模板，将编译器视作翻译器，将性能视作终极衡量。于是语言不断追赶架构，程序不断迎合硬件，抽象层在速度的恐惧中被碾碎。

Nuis 并不认为这是必然。

第五代语言不是更快的语言，而是更清醒的语言。它不为设备编程，而为意义编程；不再追逐执行模型，而是构建执行语义；不再围绕指令，而围绕意图。

Nuis 的目标不是适配未来硬件，而是让未来硬件适配它。

⸻

Ⅰ. 世界观 · Nuis 的语言哲学

Nuis 构建于三条稳定轴线上：

维度	定义	属性
语义稳定性	描述“做什么”的层永久不应被硬件侵蚀	永久稳定
执行可演化性	执行路径允许随时代替换	可替换
策略可学习性	编译决策具备智能进化能力	可演化

Nuis 认为语言并非“指令的便利书写方式”，而是“意义的承载系统”。

因此我们拒绝以下假设：
	•	指令是语言的核心
	•	性能凌驾于语义
	•	编译器只是规则机器

并确立以下信条：
	1.	用户只描述意图，执行由系统决定。
	2.	抽象必须建立在语义结构之上，而非语法糖。
	3.	时间兼容性高于平台兼容性。
	4.	编译器是策略智能体，而非静态规则机。
	5.	安全是语义一致性的必然结果。

⸻

Ⅱ. 核心抽象：语义组件

mod —— 类型化计算上下文

mod 是 Nuis 的根语义单位，其本质不是模块，而是一个“计算宇宙”。

mod cpu Host { ... }
mod kernel Compute { ... }
mod shader Lighting { ... }
mod quantum QKernel { ... }

属性：
	•	独立语义空间
	•	独立类型实现
	•	明确执行域
	•	非结构嵌套，而是语义边界

mod 构成执行图中的节点，而非简单命名空间。

⸻

channel —— 跨域数据语义桥

let ch = channel<Buffer<f32>, cpu::Host, kernel::Compute>::new();
ch.send(data);
kernel::Compute.dispatch(ch);

channel 是计算域之间的语义连接，其职责是表达：
	•	数据流向
	•	生命周期边界
	•	所有权迁移
	•	同步策略

它是调度图中的边，而非通信工具。

⸻

Ⅲ. 三层 IR 架构：语义 → 调度 → 执行

Nuis 引入明确的三层 IR 模型：

Semantic IR (NIR) —— 描述意图与结构
        ↓
Schedule IR (YIR) —— 描述任务图与执行规划
        ↓
Target IR (LLVM / SPIR-V / QIR) —— 描述指令实现

NIR：语义中间表示
	•	表达“要做什么”而非“如何做”
	•	捕捉意图、上下文、生命周期、逻辑结构
	•	不关心设备、线程或并行策略

YIR：调度中间表示

YIR 是 Nuis 的执行心脏：
	•	构建任务图（DAG）
	•	描述依赖关系
	•	映射执行域
	•	编排同步结构
	•	供 Strategy Engine 操作

这是新一代编译模型的核心，而非语法糖附属。

Target IR

执行层 IR 面向具体平台，完全可替换，仅服从 YIR 规划。

⸻

Ⅳ. Strategy Engine · 可学习的编译智能体

Strategy Engine 位于 NIR 与 YIR 之间，负责将语义抽象转化为执行结构。

支持三种模式：

strategy:
  mode: auto
  model: ai_policy

	•	Rule-driven：规则模板
	•	AI-driven：机器学习策略
	•	Profile-driven：性能反馈优化

编译不再是固定过程，而是可学习决策链。

⸻

Ⅴ. Semantic Data Model (SDM)

三层语义数据结构：

层级	职责
Mod Context Layer	语义作用域与内存边界
Entity Layer	对象状态与语义属性
Lifetime Graph Layer	生命周期演化路径

Nuis Debug 的核心原则：

Watch the Flow, not the Bytes.

⸻

Ⅵ. 类型与上下文系统

类型在 Nuis 中服从语义而非物理表示：
	•	同名类型跨域独立实现
	•	泛型是语义模板
	•	多态是意图分化

mod physics(cpu) {
    struct Vec3 { x: f32, y: f32, z: f32 }
}

mod shader(shader) {
    struct Vec3 { x: f32, y: f32, z: f32 }
}


⸻

Ⅶ. 安全模型：语义一致性安全

Nuis 的安全不是内存限定，而是语义一致性：
	•	异构 Borrow Checker
	•	生命周期图验证
	•	循环引用拒绝
	•	跨域一致性检查

⸻

Ⅷ. Runtime · 最小职责执行层

⸻

Ⅷ-A. Nuis Toolchain & LanguageOS 架构

Nuis 并非单一编译器，而是一个面向语义计算的 LanguageOS 级工具链系统，其职责是构建、维护并调度整个语义世界。

核心组件如下：

🧠 nuis-rc — Semantic Core Controller

语义运行控制中枢，负责：
	•	语义实体注册与追踪
	•	生命周期一致性维护
	•	跨域资源调度协调
	•	Strategy Engine 与 Runtime 的通信桥梁

它并非传统 runtime，而是“语义守护中枢”。

⸻

🌌 Nustar — Domain & Capability Registry

领域注册系统，用于声明与管理：
	•	新硬件域（CPU / GPU / Quantum / NPU）
	•	执行能力模型
	•	资源约束与特性描述
	•	Plugin 领域扩展

它使 Nuis 拥有可扩展的”语义宇宙”。

⸻

🧵 Nurs — Physical Bridge Layer

物理桥层，负责：
	•	与底层驱动接口交互
	•	设备初始化与生命周期控制
	•	DMA / UVM / Shared Memory 管理
	•	安全执行通道维护

它是语义世界与物理世界之间的接口层。

⸻

🛠️ nuis-cli & nuis-build

开发者工具：
	•	模块编译与调度预览
	•	语义冲突检测
	•	生命周期图可视化
	•	执行策略模拟

这些工具构成开发者与 LanguageOS 的交互窗口。

⸻

工具链协作模型

Source (.ns)
 ↓
Nuis Frontend
 ↓
Semantic IR (NIR)
 ↓
Strategy Engine
 ↓
Schedule IR (YIR)
 ↓
LanguageOS Core (nuis-rc)
 ↓
Nurs → Hardware

Nuis 工具链不是从代码到机器，而是从意图到执行文明。

运行时承担：
	•	执行设备调度
	•	Channel 同步
	•	策略模型加载
	•	性能反馈采集

其角色是执行服从者而非规则主宰者。

⸻

Ⅸ. AI 与语义共治

AI 在 Nuis 中非加速器，而是决策参与者：
	•	Semantic Advisor 提供结构分析
	•	Strategy Engine 调节执行策略
	•	反馈闭环训练

⸻

Ⅹ. Evolution Roadmap

阶段	目标	状态
v0.3	NIR 完善	✅
v0.4	MLIR Dialect	🚧
v0.5	Strategy 原型	🚧
v0.6	多设备 runtime	⏳
v0.7	Quantum backend	⏳
v1.0	语义规范冻结	🏁


⸻

终章 · 语言的未来不是更快，而是更清晰

语言若只追逐性能，终将被硬件抛弃；
语言若能守住语义，便可跨越时代。

Nuis 不为平台而生，而为意义而生。

它不是语言的延续，而是语言的反思。

当计算终于理解意图，它才真正步入文明。

⸻

License: MIT
Repository: github.com/Team-silvortex/nuislang