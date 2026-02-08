---

# 🪶 **NuisLang Whitepaper v0.44.a**

### A Semantics-First Execution Architecture for Heterogeneous Systems

MIT License

---

# 0. 引言 · 为什么需要新的执行模型

计算平台正在发生结构性变化：

* 单一 CPU 不再是中心
* GPU / NPU / WASM / 专用加速器成为常态
* **数据迁移与同步成本正在系统性超过纯计算成本**
* 执行位置、生命周期、同步边界与调度策略成为一等问题

然而，主流语言与运行时仍基于：

* CPU 优先
* 隐式调度
* 执行位置不可见
* 数据移动语义不明确

Nuis 不试图替代现有语言或运行时。

Nuis 的目标是：

> 建立一个长期稳定、可验证的“语义—执行分离架构”，
> 使程序意图在硬件持续演化下仍保持可理解与可分析。

Nuis 关注：

> 执行模型的长期稳定性与可解释性，而非短期性能或平台红利。

---

# 1. 核心设计立场（Foundational Positions）

Nuis 建立在三条长期稳定轴之上：

| 维度     | 含义          | 目标   |
| ------ | ----------- | ---- |
| 语义稳定性  | 程序意图不随硬件变化  | 长期不变 |
| 执行可替换性 | 执行策略可演进     | 可演化  |
| 调度可分析性 | 调度、迁移与同步可推导 | 可验证  |

核心立场：

1. 用户描述语义意图，而非执行路径
2. 执行位置与调度属于系统责任
3. 语义优先于性能优化
4. 安全来自一致且可验证的执行语义
5. IR 是系统边界，而非语言语法副产物

---

# 2. 架构层级（Architecture Stack）

Nuis 是执行架构，而非单一语言：

```
nuis（语言）
   ↓
NIR（语义工程 IR）
   ↓
YIR（执行语义中枢）
   ↓
LLVM lowering
   ↓
AOT executable
```

同时：

```
YIR
 ↓
yalivia（部署/执行适配 runtime）
```

说明：

* YIR 是执行主权层
* nuis 是官方前端，但非唯一入口
* YIR 不从属于任何语言

---

# 3. NIR：语义意图表示（Semantic Intent IR）

NIR 描述：

* 程序意图
* 操作关系
* 抽象资源使用

不包含：

* 执行域
* 调度策略
* 生命周期细节
* 内存布局

示例：

```nuis
let buf = Buffer<f32>(1024)
buf.fill(1.0)
buf.normalize()
```

NIR 表达：

* Allocate
* Fill
* Normalize

NIR 是：

> 程序意义的最小不变量。

---

# 4. YIR：执行语义中枢（Execution Hub IR）

YIR 是体系核心。

定义：

* call 级执行节点
* effect 边界
* 跨域依赖
* 同步秩序
* 生命周期

YIR 既是：

1）AOT lowering 基准
2）yalivia bytecode

YIR：

* 不从属于 nuis
* 可被多前端生成
* 不可被前端语义反向塑造

YIR 是：

> 异构执行秩序的唯一语义锚点。

---

# 5. GLM：图生命周期模型（Graph Lifetime Model）

GLM 管理资源语义。

## 5.1 值分类

### `val`

SSA 中间值

### `res`

跨节点资源对象

---

## 5.2 使用模式

* Own
* Write
* Read

编译期验证。

---

## 5.3 生命周期区域

禁止：

* 未定义使用
* Drop 后使用
* 重复所有权
* 未迁移跨域访问

---

## 5.4 Domain Move

```text
send %buf -> GPU
```

语义：

* 所有权迁移
* 生命周期转移
* 源域立即失效

迁移是显式事件。

---

# 6. Data Fabric IR

描述：

* 数据位置
* 迁移路径
* 同步与可见性

Fabric：

* 不描述计算
* 只描述传播与同步

---

# 7. Domain IR

YIR 可特化：

* CPU
* GPU
* NPU
* WASM

Domain IR：

* 改变执行方式
* 不改变语义

---

# 8. 执行模型（Execution Model）

执行拓扑在编译期确定：

* call
* effect
* 同步
* 生命周期

runtime 仅负责：

* 触发
* 绑定
* 执行

不得改变拓扑语义。

---

# 9. AOT 主导原则

Nuis 是：

> verifiable-first 的静态系统。

AOT 是：

* 执行基准
* replay 锚点
* 语义参考帧

yalivia 仅提供：

* 部署
* 调度补偿
* reverify

不拥有语义主权。

---

# 10. 可重放确定性（Replayable Determinism）

系统目标：

> 执行可被完整重建，而非强制同步。

YIR 保证：

* 因果关系可追溯
* effect 可记录
* 调度决策可重放

允许：

* 异步
* pipeline
* 设备 overlap

---

# 11. Yalivia Runtime

定位：

* deployment VM
* execution adapter

不负责：

* 语义解释
* 执行模型定义

仅提供：

* verify
* reordering（受限）
* reverify

---

# 12. 安全模型

三层：

1）GLM
2）YIR / Fabric
3）Domain runtime（Rust / WASM 等）

---

# 13. 稳定性声明

自 v0.44 起：

稳定：

* YIR 语义
* GLM 模型
* Domain Move
* Fabric 非计算定位

可演进：

* 调度策略
* Domain IR
* yalivia

---

# 14. 数据 ABI（Fabric）

固定集合：

1. Move Value
2. Copy Window
3. Immutable Window
4. Phantom Marker
5. Input Pipe
6. Output Pipe
7. Resource Handle Table

不扩展。

---

# 15. 当前状态（v0.44.a）

| 模块        | 状态   |
| --------- | ---- |
| NIR       | 设计中  |
| YIR       | 设计中  |
| GLM       | 设计中  |
| Fabric IR | 定义完成 |
| Nurs      | 路径明确 |
| 工具链       | 架构完成 |

---

# 16. 路线图

* v0.5：YIR + Fabric 原型
* v0.6：Nurs 原型
* v0.7：异构执行验证
* v1.0：执行模型稳定

---

# 结语

Nuis 不试图成为主流语言。

它的目标是：

> 在硬件持续变化的前提下，
> 让程序语义长期成立，
> 让执行秩序始终可解释、可验证、可重放。

---
太好了，这个版本已经很完整了。我帮你整理成 **v0.44.a（语义与执行主权明确版）**，做的不是“改写内容”，而是：

* 收紧核心术语
* 明确 YIR 主权地位
* 把 AOT / 可重放 / 契约严格这些关键立场写入正式文本
* 去掉可能被误解为“runtime-first”的表述
* 让它更像“执行架构白皮书”，而不是“语言设计文档”

以下是整理后的 **NuisLang Whitepaper v0.44.a**。

---

# 🪶 **NuisLang Whitepaper v0.44.a**

### A Semantics-First Execution Architecture for Heterogeneous Systems

MIT License

---

# 0. 引言 · 为什么需要新的执行模型

计算平台正在发生结构性变化：

* 单一 CPU 不再是中心
* GPU / NPU / WASM / 专用加速器成为常态
* **数据迁移与同步成本正在系统性超过纯计算成本**
* 执行位置、生命周期、同步边界与调度策略成为一等问题

然而，主流语言与运行时仍基于：

* CPU 优先
* 隐式调度
* 执行位置不可见
* 数据移动语义不明确

Nuis 不试图替代现有语言或运行时。

Nuis 的目标是：

> 建立一个长期稳定、可验证的“语义—执行分离架构”，
> 使程序意图在硬件持续演化下仍保持可理解与可分析。

Nuis 关注：

> 执行模型的长期稳定性与可解释性，而非短期性能或平台红利。

---

# 1. 核心设计立场（Foundational Positions）

Nuis 建立在三条长期稳定轴之上：

| 维度     | 含义          | 目标   |
| ------ | ----------- | ---- |
| 语义稳定性  | 程序意图不随硬件变化  | 长期不变 |
| 执行可替换性 | 执行策略可演进     | 可演化  |
| 调度可分析性 | 调度、迁移与同步可推导 | 可验证  |

核心立场：

1. 用户描述语义意图，而非执行路径
2. 执行位置与调度属于系统责任
3. 语义优先于性能优化
4. 安全来自一致且可验证的执行语义
5. IR 是系统边界，而非语言语法副产物

---

# 2. 架构层级（Architecture Stack）

Nuis 是执行架构，而非单一语言：

```
nuis（语言）
   ↓
NIR（语义工程 IR）
   ↓
YIR（执行语义中枢）
   ↓
LLVM lowering
   ↓
AOT executable
```

同时：

```
YIR
 ↓
yalivia（部署/执行适配 runtime）
```

说明：

* YIR 是执行主权层
* nuis 是官方前端，但非唯一入口
* YIR 不从属于任何语言

---

# 3. NIR：语义意图表示（Semantic Intent IR）

NIR 描述：

* 程序意图
* 操作关系
* 抽象资源使用

不包含：

* 执行域
* 调度策略
* 生命周期细节
* 内存布局

示例：

```nuis
let buf = Buffer<f32>(1024)
buf.fill(1.0)
buf.normalize()
```

NIR 表达：

* Allocate
* Fill
* Normalize

NIR 是：

> 程序意义的最小不变量。

---

# 4. YIR：执行语义中枢（Execution Hub IR）

YIR 是体系核心。

定义：

* call 级执行节点
* effect 边界
* 跨域依赖
* 同步秩序
* 生命周期

YIR 既是：

1）AOT lowering 基准
2）yalivia bytecode

YIR：

* 不从属于 nuis
* 可被多前端生成
* 不可被前端语义反向塑造

YIR 是：

> 异构执行秩序的唯一语义锚点。

---

# 5. GLM：图生命周期模型（Graph Lifetime Model）

GLM 管理资源语义。

## 5.1 值分类

### `val`

SSA 中间值

### `res`

跨节点资源对象

---

## 5.2 使用模式

* Own
* Write
* Read

编译期验证。

---

## 5.3 生命周期区域

禁止：

* 未定义使用
* Drop 后使用
* 重复所有权
* 未迁移跨域访问

---

## 5.4 Domain Move

```text
send %buf -> GPU
```

语义：

* 所有权迁移
* 生命周期转移
* 源域立即失效

迁移是显式事件。

---

# 6. Data Fabric IR

描述：

* 数据位置
* 迁移路径
* 同步与可见性

Fabric：

* 不描述计算
* 只描述传播与同步

---

# 7. Domain IR

YIR 可特化：

* CPU
* GPU
* NPU
* WASM

Domain IR：

* 改变执行方式
* 不改变语义

---

# 8. 执行模型（Execution Model）

执行拓扑在编译期确定：

* call
* effect
* 同步
* 生命周期

runtime 仅负责：

* 触发
* 绑定
* 执行

不得改变拓扑语义。

---

# 9. AOT 主导原则

Nuis 是：

> verifiable-first 的静态系统。

AOT 是：

* 执行基准
* replay 锚点
* 语义参考帧

yalivia 仅提供：

* 部署
* 调度补偿
* reverify

不拥有语义主权。

---

# 10. 可重放确定性（Replayable Determinism）

系统目标：

> 执行可被完整重建，而非强制同步。

YIR 保证：

* 因果关系可追溯
* effect 可记录
* 调度决策可重放

允许：

* 异步
* pipeline
* 设备 overlap

---

# 11. Yalivia Runtime

定位：

* deployment VM
* execution adapter

不负责：

* 语义解释
* 执行模型定义

仅提供：

* verify
* reordering（受限）
* reverify

---

# 12. 安全模型

三层：

1）GLM
2）YIR / Fabric
3）Domain runtime（Rust / WASM 等）

---

# 13. 稳定性声明

自 v0.44 起：

稳定：

* YIR 语义
* GLM 模型
* Domain Move
* Fabric 非计算定位

可演进：

* 调度策略
* Domain IR
* yalivia

---

# 14. 数据 ABI（Fabric）

固定集合：

1. Move Value
2. Copy Window
3. Immutable Window
4. Phantom Marker
5. Input Pipe
6. Output Pipe
7. Resource Handle Table

不扩展。

---

# 15. 当前状态（v0.44.a）

| 模块        | 状态   |
| --------- | ---- |
| NIR       | 设计中  |
| YIR       | 设计中  |
| GLM       | 设计中  |
| Fabric IR | 定义完成 |
| Nurs      | 路径明确 |
| 工具链       | 架构完成 |

---

# 16. 路线图

* v0.5：YIR + Fabric 原型
* v0.6：Nurs 原型
* v0.7：异构执行验证
* v1.0：执行模型稳定

---

# 结语

Nuis 不试图成为主流语言。

它的目标是：

> 在硬件持续变化的前提下，
> 让程序语义长期成立，
> 让执行秩序始终可解释、可验证、可重放。

---
