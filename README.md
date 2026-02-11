---

# 🪶 **NuisLang Whitepaper v0.44.b**

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
2. 执行位置与调度属于系统责任（编译器与工具链层）
3. 语义优先于性能优化
4. 安全来自一致且可验证的执行语义
5. IR 是系统边界，而非语言语法副产物

说明：

“系统责任”指 **nuisc + lowering toolchain** 的编译期职责，而非 runtime 的自主行为。
runtime 不拥有执行拓扑主权。

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

## 2.1 Toolchain Roles（工具链分权）

```
nuis source
   ↓
nuisc
   - 执行拓扑构造
   - 调度约束编译
   - 契约检查
   - YIR 生成
   ↓
nustar (per-mod)
   - mod语法
   - 解析
   - lowering
   - mod AST
   ↓
YIR
```

nuisc：

* 不负责硬件语法解析
* 不直接做 target lowering
* 负责 execution topology 与 contract

nustar：

* 提供 mod 语法/语义/能力
* 生成 mod AST
* 必须服从统一规范与接口
* 不得改变 YIR 核心语义

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

（保持原文）

---

# 6. Data Fabric IR

Fabric：

* 只描述数据迁移与同步
* 不包含计算语义
* 不允许引入 execution op

---

# 7. Domain IR

Domain IR：

* 改变执行方式
* 不改变语义
* 不引入新的 effect 类型
* 不改变 GLM 规则

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

## 8.1 Topology-First Principle

程序首先定义 **计算拓扑**：

* 执行关系图
* effect 结构
* 生命周期流

调度与数据行为存在 runtime 维度，但必须：

* 服从拓扑
* 可重放
* 不改变语义

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

## 9.1 Execution Contract & Fail-Fast

AOT executable 建立 **执行契约**：

包含：

* 可用 domain
* 资源假设
* YIR 版本
* Fabric ABI
* GLM 约束

契约不成立：

> 程序必须拒绝执行。

禁止：

* fallback
* 模拟
* 自动降级

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

## 10.1 Trace Requirements

重放最小记录：

* effect 序
* Domain Move
* Fabric 事件
* 资源句柄映射
* 调度决策（若存在）

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

## 11.1 Optional JIT Domain

JIT：

* 完全可选
* 默认纯 AOT

动态逻辑：

* 必须拆为独立 JIT module
* 由 yalivia 执行
* 与 AOT 通过标准协议通信

禁止：

* 共享内存直接交互
* runtime 修改拓扑

---

# 12. 安全模型

（保持）

---

# 13. 稳定性声明

（保持）

---

## 13.1 Mod Registry & Conformance

mod 接入：

* 通过 nustar 注册
* 必须通过 conformance 验证

nuisc 有权：

> 拒绝语义不一致的 mod。

官方维护主流硬件 mod。

---

# 14. 数据 ABI（Fabric）

（保持）

---

# 15. 当前状态（v0.44.b）

| 模块        | 状态   |
| --------- | ---- |
| NIR       | 设计中  |
| YIR       | 设计中  |
| GLM       | 设计中  |
| Fabric IR | 定义完成 |
| nuisc     | 架构完成 |
| nustar    | 路径明确 |

---

# 16. 路线图

（保持）

---

# 结语

Nuis 不试图成为主流语言。

它的目标是：

> 在硬件持续变化的前提下，
> 让程序语义长期成立，
> 让执行秩序始终可解释、可验证、可重放。
