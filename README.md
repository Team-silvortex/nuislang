# nuislang

> AOT-first heterogeneous systems language and toolchain built around `nuis -> NIR -> YIR -> LLVM/AOT`, with `nustar` packages providing per-domain parsing, lowering, ABI contracts, and verification surfaces.

## Current Status

The repository is in an active architecture-building stage. The most stable current spine is:

```text
nuis source / project
  -> nuisc
  -> NIR
  -> YIR
  -> LLVM / AOT packaging
```

The key thing that is already real today is not “all language features are done”, but that the project now has one increasingly consistent execution model across:

* `cpu`
* `data`
* `shader`
* `kernel`

That model is increasingly enforced through `YIR` contracts, project validation, per-domain `nustar` manifests, and verifier checks rather than only ad hoc frontend rules.

## Toolchain

```text
nuis     -> front-door workflow tool
nuis-rc  -> resident control tool (later-stage, still intentionally thin)
nuisc    -> compiler/scheduler core
yalivia  -> separate future JIT/runtime project
vulpoya  -> separate future analyzer/verifier project
```

Current responsibility split:

* `nuis` is the main workflow surface for `check`, `build`, caches, projects, and package inspection.
* `nuisc` is the compiler/scheduler core that consumes `.ns` or project inputs and emits `NIR`, `YIR`, LLVM IR, and AOT outputs.
* `nustar` packages are where per-domain ABI support, default lanes, frontend/lowering entrypoints, and package contracts are registered.

## Quick Start

Recommended first commands:

```bash
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
```

Useful inspection commands:

```bash
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
```

CPU target override is now explicit:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project

cargo run -p nuis -- build --target aarch64-apple-darwin \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project
```

## What Is Working Well Right Now

High-signal implemented surfaces:

* multi-file `nuis.toml` projects with project-level `links`
* lazy `nustar` loading and per-domain ABI resolution
* ABI auto-selection from registered `abi_targets`
* explicit `--cpu-abi` and `--target` overrides for CPU builds
* compile-cache inspection and pruning through `nuis`
* AOT bundle generation for current CPU-only and macOS window-hosted demo paths
* project-level host FFI contract indexing
* `cpu/data/shader/kernel` result-family validation in `YIR`
* task-style async primitives with `spawn / join / cancel / timeout / join_result`
* core-level async/result metadata beginning to move into `yir-core`

## Current Reference Examples

Start here:

* [examples/README.md](/Users/Shared/chroot/dev/nuislang/examples/README.md)
* [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)
* [examples/bins/README.md](/Users/Shared/chroot/dev/nuislang/examples/bins/README.md)
* [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)

Recommended current examples:

* [examples/ns/core/hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
* [examples/ns/types/hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)
* [examples/ns/data/hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
* [examples/ns/ffi/hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
* [examples/ns/demos/window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
* [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [examples/projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* [examples/yir/demos/window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
* [examples/yir/data/data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_fabric_demo.yir)
* [examples/yir/shader/shader_bindings_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_bindings_demo.yir)
* [examples/yir/kernel/kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_auto_broadcast_demo.yir)

## Key Architectural Notes

Current high-signal architectural facts:

* `YIR` is the main semantic execution boundary in this repository.
* `nuisc` is intentionally becoming more mod-agnostic; per-domain support should come from registered `nustar` contracts.
* `abi_targets` now live in `nustar` manifests and drive auto ABI selection, CLI overrides, packaging metadata, and loader contracts.
* default lane policy also belongs to `nustar` manifests; `nuisc` should only apply that policy plus narrow fallbacks.
* `data.handle_table` remains an indirection/resource-binding surface, not a place to own large payload blobs directly.
* current Fabric host integration is intentionally thin and AOT-first, with static typed action tables rather than a heavy runtime metadata graph.
* async/result semantics are being normalized into `yir-core`, even though the concrete entry ops are still currently surfaced through `cpu.*`.

## Notes

This README is still paired with the longer architecture/whitepaper material below. The whitepaper remains useful for long-range design direction, but the sections above should be treated as the faster-moving “what is actually true in this repo right now” layer.

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
2. 执行位置与调度属于系统责任
3. 语义优先于性能优化
4. 安全来自一致且可验证的执行语义
5. IR 是系统边界，而非语言语法副产物

**补充说明（v0.44.b）**
这里的“系统责任”指 **编译器与工具链层（nuisc + lowering toolchain）** 的职责，而非 runtime 在运行时临场做“自主智能调度”。
runtime 可以执行、绑定、触发，但 **不得拥有执行拓扑主权**、不得反向塑造 YIR 语义。

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
yalivia（独立项目：部署/执行适配 runtime）
```

说明：

* YIR 是执行主权层
* nuis 是官方前端，但非唯一入口
* YIR 不从属于任何语言

---

## 2.1 Toolchain Roles（工具链分权，v0.44.b）

Nuis 的“执行架构”不仅是 IR 分层，也是 **编译职责分层**：

```
nuis source
   ↓
nuisc（核心编译器：执行调度编译）
   ↓
nustar（per-mod：语法/解析/lowering）
   ↓
mod AST
   ↓
NIR / YIR
   ↓
AOT executable / external yalivia integration
```

### nuisc（Execution Scheduler Compiler）

nuisc 的职责是 **调度与执行拓扑编译**：

* 构造执行拓扑（call / effect / dep / lifetime）
* 进行契约检查（Execution Contract）
* 生成并验证 YIR（作为语义锚点）
* 选择 AOT/JIT profile 的编译模式与产物形态

nuisc **不负责**：

* 各硬件/Domain 的专属语法解析
* 具体 target 的 lowering 细节实现

### nustar（Per-Mod Frontend + Lowering）

nustar 是硬件/Domain 的“能力注入点”：

* 提供 mod 专属语法与解析器
* 产出 mod AST
* 完成 lowering（将 mod AST 降到 NIR / YIR 可接受的形态）
* 必须服从统一定义规范与接口规范

**关键约束**：
nustar 可以扩展语法、扩展语义、扩展能力，但它的所有产物必须 **可被 nuisc 静态验证**，且不得破坏 YIR 的核心语义锚点地位。

---

## 2.2 Profiles & External Integration（v0.44.b）

`nuis` 本身以 **AOT profile** 为主轴：

* **nuis(AOT profile)**：严格静态、可复现、无运行时调度标记

若未来需要与动态系统对接，应通过 **外部项目**（例如 `yalivia`）消费 YIR 或其兼容边界来完成，而不是把 `yalivia` 视为 `nuis` 仓库内的第二主 profile。

约束仍然成立：

* 外部运行时标记不得反向塑造 YIR 核心语义
* AOT 语义边界始终是 `nuis` 的基准参考帧

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
2）外部执行系统可消费的语义边界

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

**补充约束（v0.44.b）**
Fabric IR 是数据传播语义层，必须保持“非计算定位”：

* 禁止携带 compute op
* 禁止引入执行语义捷径（例如把 compute 包进搬运事件）
* 仅允许表达：位置、路径、同步、可见性、句柄表映射

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

**补充约束（v0.44.b）**
Domain IR 的可演进性不意味着可越界：

* 不得引入新的 effect 类型
* 不得改变 GLM 的所有权/生命周期规则
* 只能在既定语义下改变执行方式（lowering / codegen strategy）

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

## 8.1 Topology-First Principle（v0.44.b）

Nuis 的执行模型以 **计算拓扑** 为第一性对象。
编译期首先确定：

* 执行关系图（call graph / dep graph）
* effect 边界与因果关系
* 生命周期流（GLM + Domain Move）

调度与数据行为可能存在 runtime 维度，但必须满足：

* 在拓扑约束内发生
* 可记录、可重放
* 不改变拓扑语义本身

---

# 9. AOT 主导原则

Nuis 是：

> verifiable-first 的静态系统。

AOT 是：

* 执行基准
* replay 锚点
* 语义参考帧

若接入 `yalivia`，它仅提供：

* 部署
* 调度补偿
* reverify

不拥有语义主权。

---

## 9.1 Execution Contract & Fail-Fast（v0.44.b）

AOT executable 建立 **执行契约（Execution Contract）**。契约至少包含：

* 允许的 Domain 集合与版本约束
* YIR 版本与语义兼容要求
* Fabric ABI 版本
* 资源模型假设（例如：可用内存域、显存域、句柄表策略等）
* 关键能力约束（例如：某些 Domain Move 的可用性）

**契约不成立 → 程序必须拒绝执行**。
禁止：

* 自动降级
* 隐式 fallback
* 模拟执行替代真实 Domain
* “尽量跑起来”的容错路径

Fail-Fast 是 AOT-first 的必需条件：AOT 作为 replay 锚点时，必须保证执行准入严格一致。

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

## 10.1 Trace Requirements（v0.44.b）

为保证“可被完整重建”，系统需要定义最小重放记录面（minimum trace surface）：

* effect 序与边界
* Domain Move 事件序列
* Fabric 事件序列（Move/Window/Pipe/HandleTable 等）
* 资源句柄映射（Resource Handle Table 的稳定映射与版本）
* 调度决策（若存在可变调度层，则记录其决策以重放）

目标是重建因果与秩序，而非强制同步执行。

---

# 11. Yalivia（External Project Boundary）

若存在 `yalivia` 集成，其定位是：

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

## 11.1 Optional External JIT Domain（v0.44.b）

JIT 完全可选。默认系统为 **纯 AOT**，不允许共享内存的多进程动态行为污染 AOT 域。

若需要动态性，应当：

* 将相关逻辑拆为 **JIT domain module**
* 由独立项目 `yalivia` 执行
* 与 AOT 域通过 **标准通用协议** 交互（而非内部私有协议）

原则：

* AOT 域保持可复现与契约执行
* JIT 域提供灵活性与精度调优
* 两者之间 **不直接互操作内存/运行时状态**，仅通过通信与句柄/窗口/管道等 ABI 机制交互（类比 Kotlin Native/JVM 的分域互通）

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
* 外部执行适配项目（例如 yalivia）

---

## 13.1 Mod Registry & Conformance（v0.44.b）

Nuis 采用 **注册式 mod 接入**：

* 每个主流硬件/Domain 对应一组 mod（含其 nustar、mod AST、lowering）
* mod 必须通过 conformance 验证，保证其语义与接口满足统一规范

nuisc 具有最终否决权：

> 任何语义不一致、契约冲突、无法静态验证的 mod —— 必须拒绝注册。

治理原则：

* AOT-first 的语义纯度优先于生态广度
* 主流硬件的 mod 由官方实现并维护一致性
* 第三方扩展必须以 contract 与 conformance 为准入条件，而非“能跑即可”

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

# 15. 当前状态（v0.44.b）

| 模块                | 状态   |
| ----------------- | ---- |
| NIR               | 设计中  |
| YIR               | 原型可跑 |
| GLM               | 设计中  |
| Fabric IR         | 定义完成 |
| Nurs              | 路径明确 |
| 工具链               | 架构完成 |
| nuisc / nustar 分权 | 路径明确 |

---

## 15.1 Repository Alignment（仓库对齐状态，v0.44.b）

当前仓库实现仍处于**骨架阶段**，但从本版本开始，工程命名与职责边界按以下口径收敛：

* `tools/nuis`：作为工具链前门原型入口，代表用户侧 workflow command
* `tools/nuis-rc`：作为常驻控制器原型入口，负责本机上的 `nuis` 工具链版本与项目索引
* `tools/nuisc`：作为 `nuisc` 原型入口，代表执行拓扑编译器
* `crates/nuis-runtime`：仅表示本仓库内的 AOT 侧执行支撑骨架，**不代表 `yalivia`**
* `crates/nuis-semantics`：承载 NIR / YIR / Fabric / contract 的语义模型占位
* `docs/fabric-spec/DFIR.md`：文件名暂保留历史命名，但内容应以 **Fabric IR / Fabric ABI** 为准
* `nustar-packages/index.toml`：当前 `nustar` 静态索引入口，`nuisc` 通过它做惰性装载，而非主动扫目录发现
* 当前 `nuis / nuisc` 也已有最小 `bindings` 工作流：先形成当前编译图，再得到本次真正需要的 `nustar` binding plan
* 当前 `nuisc` 也已有最小 `nustar` 标准二进制包骨架：可将 manifest + implementation blob 打成统一包体，并做读取/检查
* 当前 `nustar` loader contract 也已有最小规范：统一走 `nustar-loader-v1`，canonical entry 为 `nustar.bootstrap.v1`，并带机器 ABI 兼容字段
* 当前 `cpu / shader / kernel` 三个主 `nustar` 已开始显式声明自己的职责面：具体 `AST`、面向 `nuis` 的 `NIR surface`、到 `YIR` 的 lowering，以及各自负责的 `part verify`

额外边界说明：

* `yalivia` 是**独立项目**
* `vulpoya` 是**独立项目**
* 本仓库主线是 **AOT-first 的 nuis 工具链**
* 与 `yalivia` 的关系仅是未来的外部对接边界，而不是当前仓库内部 runtime 分层
* 与 `vulpoya` 的关系仅是未来的外部语法/约束分析协作边界，而不是当前仓库内部前端分层

这意味着：

* 当前仓库**尚未实现完整 NIR / 最终 GLM / 完整 Fabric verifier**
* 已具备最小 `YIR` 手写原型：`parse -> verify -> execute`
* 当前 `YIR` 原型采用 **注册式 mod 指令集** 与**显式图边模型**
* 当前已新增工作中的 `YIR Reference`：`docs/reference/yir-reference.md`，并拆为 `docs/reference/yir-langref.md` 与 `docs/reference/yir-tools-reference.md`，用于像早期 `LLVM LangRef + tools reference` 那样同步整理现有 reference surface
* 当前已接入 `cpu` 与 backend-agnostic 的 `shader` mod；窗口/UI/present 作为 `cpu` 域特化能力存在
* 上述窗口/UI/present 仅是当前 reference preview adapter 消费的 `cpu`-mod 能力，**不是 `YIR` core 对 UI 框架的内建依赖**
* 当前 `cpu` mod 已开始覆盖 `arm64-family` 的抽象能力面（如 `target_config` / `bind_core` / `madd`）
* 当前 `cpu` mod 也已有最小条件数据流、位运算与整数基础算子原型（如 `eq / ne / lt / gt / select / and / or / xor / shl / shr / div / rem / neg / not`），用于在保持静态图结构的前提下表达更强的 CPU 语义
* 当前 `cpu` mod 也已有最小可寻址对象/指针原型（`null / borrow / borrow_end / move_ptr / alloc_node / alloc_buffer / load_* / store_* / free`），用于验证链表和 buffer 这类动态结构
* 当前 `cpu` verifier 已开始按 Rust 风格收紧所有权边界：借用指针可读不可写，所有权移动后原名不可再用，释放后借用再读会被拒绝，并支持显式 `borrow_end` 收束借用生命周期
* 当前 `YIR` verifier 也已接入最小 `GLM` 约束：`res` 的 `Own / Write` 访问需要显式 `lifetime` 图边，`cpu.move_ptr / cpu.free / store_*` 等节点开始进入显式生命周期排序
* 当前 `kernel` mod 已补最小张量计算原型（如 `tensor` / `matmul` / `add_bias` / `relu`），用于 macOS 上先行验证 `cpu <-> kernel/npu` 的异构图
* 当前 `shader` mod 已开始覆盖 `Metal/Vulkan` 共有的渲染抽象面（如 `target` / `viewport` / `pipeline` / `begin_pass` / `draw_instanced`）
* 当前已补 `shader lowering contract` 分析：`draw_instanced + begin_pass + target + pipeline` 会被标注为未来 backend lowering 子集；其余 shader reference op 目前明确走 prerender fallback
* 当前 `shader package` 也已有最小清单骨架：同一 stage 会预留 `metal / vulkan / directx / opengl` 变体槽位，以便未来按 backend cooked/package 模式加载
* 当前标准能力面已开始收敛为 `cpu / shader / kernel / data`，并以 `nustar` 注册包形态存在，manifest 位于 `nustar-packages/*.toml`
* 当前 demo 直接由 `shader mod` 驱动；后续应逐步被标准库第三层 `ns-nova` 吸收为更完整的 GPU-native 应用/渲染框架入口
* GPU lane 产出的 `FrameSurface` 已可导出为实际图像文件（PPM）以验证异构执行结果
* macOS 下已补最小系统窗口预览器骨架：CPU 侧创建窗口并展示 GPU framebuffer 导出的图像；它属于工具层 adapter，不代表 `nuis` 对 Swift/AppKit 的语义依赖
* 当前已补最小 `AOT bundle` 入口：`tools/yir-pack-aot` 会优先把 CPU slice 走 `YIR -> LLVM IR -> clang` 编成本地二进制；若模块包含异构渲染结果，则会输出 `shader_contract.txt`、`shader_package.toml`，并按当前能力额外打包预渲染 frame 资产
* 带 `cpu.tick_i64` 的窗口/控件 demo 现在也可走内嵌 runtime 的单体 AOT 路：生成的 macOS AppKit host 会静态链接 `yir-runtime-host`，把 `.yir` 模块字节直接嵌进二进制里，并在进程内持续生成/消费 framebuffer
* 当前 `nuisc` 已具备最小注册发现入口：`cargo run -p nuisc -- registry`
* 当前 `nuisc` 也已具备最小前端链：`hello_world.ns` 可走 `nuis -> NIR -> YIR -> LLVM -> arm64 binary`
* 当前 `nuisc` 前端也已拆成真正的 `AST -> NIR -> YIR` 三层，且最小 `.ns` 已支持 `let`、变量引用、括号和 `+ - * /` 表达式；前门可用 `dump-ast / dump-nir / dump-yir`
* CPU-hosted UI event demo: `examples/yir/demos/host_ui_sphere.yir`
* CPU linked-list demo: `examples/yir/cpu/cpu_linked_list.yir`
* Rust-ish CPU ownership demo: `examples/yir/cpu/cpu_linked_list_rustish.yir`
* Rust-ish CPU buffer demo: `examples/yir/cpu/cpu_buffer_rustish.yir`
* Invalid borrowed-write demo: `examples/invalid/yir/cpu_borrow_write_invalid.yir`
* Invalid borrowed-buffer-write demo: `examples/invalid/yir/cpu_buffer_borrow_write_invalid.yir`
* Invalid use-after-free demo: `examples/invalid/yir/cpu_use_after_free_invalid.yir`
* CPU/kernel tensor demo: `examples/yir/kernel/kernel_tensor_demo.yir`
* Legacy CPU/NPU tensor demo: `examples/legacy/npu_tensor_demo.yir`
* 一次性窗口入口：`bash tools/yir-preview-macos/run-ball-once.sh`
* 现有 Rust crate 主要用于固定术语、边界与后续实现入口
* 后续整改优先级应为：`YIR expand -> semantics model -> verifier hardening -> AOT executable path`

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
