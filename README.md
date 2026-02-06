---

# 🪶 **NuisLang Whitepaper v0.44**

### A Semantics-First Execution Model for Heterogeneous Systems

MIT License

---

# 0. 引言 · 为什么需要新的执行模型

计算平台正在发生结构性变化：

* 单一 CPU 不再是中心
* GPU / NPU / WASM / 专用加速器成为常态
* **数据迁移与同步成本正在系统性地超过纯计算成本**
* 执行位置、生命周期、同步边界与调度策略成为一等问题

然而，主流语言与运行时的核心假设仍停留在单域、CPU 优先、隐式调度的时代。

Nuis 的目标并非取代现有语言或框架，而是：

> 为异构计算提供一个长期稳定的“语义—执行”分离模型，
> 在不牺牲程序可理解性的前提下，系统性管理执行位置、数据流动与同步秩序。

Nuis 关注的是：

> 执行模型的可持续性与可分析性，而非短期性能或平台红利。

---

# 1. 设计原则（Design Principles）

Nuis 建立在三条长期稳定轴之上：

| 维度     | 含义            | 目标   |
| ------ | ------------- | ---- |
| 语义稳定性  | 程序“意图”不随硬件变化  | 长期不变 |
| 执行可替换性 | 执行策略可随时代演进    | 可演化  |
| 调度可分析性 | 调度、迁移与同步行为可推导 | 可验证  |

核心原则：

1. 用户描述意图，而非执行路径
2. 执行位置与调度是系统责任，而非用户负担
3. 语义优先于性能优化
4. 安全来自一致、可验证的语义模型
5. IR 是系统边界，而非语言语法的副产物

---

# 2. NIR：语义意图表示（Semantic Intent IR）

NIR（Nuis Intent Representation）是最高层表示，仅描述：

* 程序语义意图
* 操作之间的逻辑关系
* 抽象资源的使用方式

不包含：

* 执行域
* 生命周期细节
* 调度策略
* 内存布局

示例：

```nuis
let buf = Buffer<f32>(1024)
buf.fill(1.0)
buf.normalize()
```

NIR 表示：

* Allocate(1024)
* Fill(1.0)
* Normalize()

NIR 是程序意义的最小不变量。

---

# 3. YIR：跨域调度表示（Cross-Domain Scheduling IR）

YIR 是核心执行表示，用于统一描述：

* 计算节点
* 数据与控制依赖
* 执行域映射
* 同步边界
* 生命周期边界

YIR：

* 不是纯数据流图
* 支持控制节点与同步节点
* 调度层面可规约为 DAG 用于分析

目标：

> 在异构执行条件下保持执行秩序的可解释性与可预测性。

---

# 4. GLM：图生命周期模型（Graph Lifetime Model）

GLM 定义 YIR 的资源与生命周期语义。

## 4.1 值分类

### `val`

* SSA 中间值
* 不进入生命周期分析

### `res`

* 资源对象
* 可跨节点与执行域
* 必须受 GLM 管理

---

## 4.2 使用模式（UseMode）

* Own：唯一所有权
* Write：独占写
* Read：共享读

约束：

* 同时仅允许一个 Own
* Write 不并发
* Read 可并发

编译期验证。

---

## 4.3 生命周期区域（Region）

非法情形：

* 定义前使用
* Drop 后使用
* 重复所有权
* 未迁移即跨域访问

GLM 的目标：

> 在编译期消除跨域资源使用中的未定义行为。

---

## 4.4 跨域迁移（Domain Move）

```text
send %buf -> GPU
```

表示：

* 所有权迁移
* 生命周期扩展
* 源域立即失效

迁移是显式语义事件。

---

# 5. Data Fabric IR：数据传播与同步平面

Data Fabric IR 描述：

* 数据位置
* 迁移路径
* 同步与可见性

不是计算 IR。

---

## 5.1 设计定位

描述：

* memory space
* copy / map / peer transfer
* event / fence / token

---

## 5.2 与 CPU 模块的分界

CPU 模块内部：

* 普通函数执行
* 不强制 dataflow

Fabric 边界：

* 显式进入迁移与同步语义

---

# 6. Domain IR：执行域特化

YIR 可特化为：

* CPU IR
* GPU IR
* NPU IR
* WASM IR

Domain IR 仅定义如何执行，不改变语义。

---

# 7. Nurs：YIR-CPU ↔ Rust MIR 映射

* 无 C ABI
* 无绑定层
* 语义对齐

Rust 是可替换执行后端之一。

---

# 8. 执行模型（Execution Model）

执行图在编译期确定：

* 节点
* 迁移
* 同步
* 生命周期

runtime 负责：

* 执行驱动
* 触发
* 资源绑定

不改变执行拓扑。

---

# 9. Fabric Core

Fabric 负责：

* 数据 transport
* 同步传播
* 可见性保证

不负责：

* 计算
* 数据生产
* 数据消费

---

# 10. Yalivia Runtime（可选）

用于：

* 实验性调度
* 多语言接入
* JIT 执行

不属于核心语义。

---

# 11. 安全模型

三层：

1. GLM：资源一致性
2. YIR / Fabric：同步可验证
3. 执行域安全（Rust / WASM 等）

---

# 12. 稳定性声明（Stability Declaration）

v0.44 起：

以下语义视为稳定：

* NIR 语义
* YIR 基本结构
* GLM 使用模型
* Domain Move 显式语义
* Fabric 非计算定位

可演进：

* Domain IR
* 调度策略
* Yalivia

---

# 13. 数据 ABI（Fabric）

Fabric 支持固定数据范式：

1. Move Value
2. Copy Window
3. Immutable Window
4. Phantom Marker
5. Input Pipe
6. Output Pipe
7. Resource Handle Table

该集合不扩展。

---

# 14. 当前状态（v0.44）

| 模块             | 状态   |
| -------------- | ---- |
| NIR            | 设计中   |
| YIR            | 设计中   |
| GLM            | 设计中   |
| Data Fabric IR | 定义完成 |
| Nurs           | 路径明确 |
| 工具链            | 架构完成 |

---

# 15. 路线图

* v0.5：YIR + Fabric 原型
* v0.6：Nurs 原型
* v0.7：异构调度验证
* v1.0：执行模型稳定发布

---

# 结语

Nuis 不试图成为新的主流语言。

它的目标是：

> 在硬件持续变化的前提下，
> 让程序语义长期成立，
> 让执行秩序始终可解释。
