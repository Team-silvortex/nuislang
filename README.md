---

# 🪶 **NuisLang Whitepaper v0.41**

### A Language Designed to Outlive Hardware.

**MIT License**

---

# 0. 序 · 语义文明的宣言

当计算以指令为中心，语言便服务于机器；
当计算以语义为中心，语言才服务于文明。

传统语言的演化，被迫围绕硬件不断重塑自身：
为性能妥协、为指令集让步、为架构杂糅抽象。

**Nuis 拒绝这一宿命循环。**

Nuis 是第五代语义语言：
**意图优先、语义优先、调度优先。**

Nuis 不为硬件而弯曲，而让硬件为语义而生。
Nuis 不是被时代塑造的语言，而是塑造时代的语言。

---

# 1. Nuis 的世界观（Philosophy）

Nuis 设计建立于三条“长期稳定轴”之上：

| 维度         | 定义           | 属性   |
| ---------- | ------------ | ---- |
| **语义稳定性**  | “做什么”不受硬件影响  | 永久稳定 |
| **执行可演化性** | 执行路径随时代替换    | 可替换  |
| **策略可学习性** | 编译器具备自适应策略智能 | 可演化  |

**设计信条**

1. 用户描述意图，执行策略应由系统决定。
2. 抽象以语义为根，不依赖语法表象。
3. 时间兼容性优先于平台兼容性。
4. 编译器是一种策略智能体，而非规则堆叠机。
5. 安全来自语义一致性，而非外加限制。

---

# 2. NIR：语义意图 IR（Semantic Intent IR）

NIR（Nuis Intent Representation）是 Nuis 的最高层语义表达。

它捕获：

* **意图（Intent）**
* **逻辑结构**
* **资源抽象**
* **跨平台的不变量**

在 NIR 中，程序不是指令流，而是 **语义节点的关系图**。

示例：

```nuis
let buf = Buffer<f32>(1024)
buf.fill(1.0)
buf.normalize()
```

其 NIR 形式：

* Allocate(1024)
* Fill(1.0)
* Normalize()

**NIR 不含任何执行域信息，不含生命周期，不含调度策略。**
它是“程序意义”的纯粹化。

---

# 3. YIR：跨域调度 IR（Cross-Domain Scheduling IR）

YIR 是整个 Nuis 体系的中枢，也是真正的执行真理层。

YIR 负责：

* CPU / GPU / Shader / WASM 的跨域调度
* Dataflow + Controlflow 的统一表示
* 资源图（Resource Graph）
* 生命周期图（GLM）
* 后端选择与域映射
* Rust / WASM / GPU / Yalivia 的衔接

**每个 YIR 程序是一个 DAG（有向无环图）**：

* Node：计算单元
* Edge：数据 / 控制依赖
* Res：跨 Node 的资源对象

YIR = “NIR 的语义” → “物理执行的分布式图”。

---

# 4. YIR 内存模型 v0.41（GLM Memory Model）

GLM（Graph Lifetime Model）是 YIR 的正式生命周期语义。

## 4.1 值分类（Value Classes）

### (1) `val`

短生命周期 SSA 值

* 中间结果
* 不跨 Node
* 不进入 GLM

### (2) `res`

资源型对象

* Buffer / Image / Handle / Tensor
* 跨域迁移
* 跨图存在
* 必须进入 GLM 分析

---

## 4.2 资源使用模式（UseMode）

对每个 `res`：

* **Own**：节点拥有资源（可 drop / move）
* **Write**：可变借用（唯一）
* **Read**：只读（可并发）

规则：

* 任何时刻 **同一资源只能存在一个 Own**
* Write 不得与 Read/Write 并发
* 多 Read 可并发（无 Write）

---

## 4.3 生命区（Region）

Region(R) = res R 的全域合法使用图。

构建：

1. 找到定义节点 `N_def`
2. 收集所有 `Use(R)`
3. 包含必要控制路径
4. `Drop(R)` 截断生命周期

非法情形：

* 定义前使用
* Drop 后使用
* 双重所有权
* Write 冲突
* 跨域悬挂（未迁移先使用）

---

## 4.4 跨域移动（Domain Move）

示例：

```
send %buf -> GPU
```

语义：

* Own 迁移到 GPU
* CPU 侧不再拥有 buf
* GLM 扩展 Region 覆盖目标域

跨域移动是 GLM 的关键节点。

---

## 4.5 完整示例

```
N1: alloc  %buf (CPU)      ; Own
N2: write  %buf (CPU)
N3: send   %buf -> GPU     ; move Own
N4: read   %buf (GPU)
N5: read   %buf (GPU)
N6: send   %buf -> CPU     ; move Own back
N7: read   %buf (CPU)
N8: drop   %buf (CPU)
```

Region(%buf) = {N1..N8}

N8 后使用 → 编译期报错。

---

# 5. Domain IR（物理特化层）

Domain IR 是 YIR 的物理域特化，包含：

### 5.1 CPU Domain IR（CIR）

* MIR 等价层
* 显式控制流
* 显式 Drop
* 由 Rust/LLVM 执行

### 5.2 GPU Domain IR（GIR）

* 结构化 SPIR-V / Compute 语义
* 线程网格 / block / subgroup
* 目标后端：Vulkan/Metal/CUDA

### 5.3 Shader Domain IR（SDFG）

* 图形 Shader 特化
* 材质、渲染流水线语义
* 与 GIR 并列但专注图形

---

# 6. Nurs：YIR-CPU ↔ Rust MIR 双向桥

**Nurs = Semantic Bridge between YIR-CPU and Rust MIR**

理由：

* 两者均为 SSA + CFG 结构
* Resource 语义由 GLM 统一
* MIR 与 YIR-CPU 高度同构
* 可实现 Lowering 与 Lifting

### 6.1 Nuis → Rust 路径

```
NIR → YIR → YIR-CPU
 → Nurs Lowering → MIR → LLVM → Native
```

### 6.2 Rust → Nuis 路径

```
Rust AST → MIR → Nurs Lifting → YIR-CPU
```

无需 C ABI，无需 bindgen。
Nuis 与 Rust 在 CPU 层是平齐的兄弟 IR。

### 6.3 稳定性

* MIR 不稳定
* YIR 为主轴
* Nurs 适配 MIR 演化
* 最终可能反向影响 MIR 模式

---

# 7. Go 的角色：Yalivia 的 一级扩展语言

Go 的定位：

* 通过 Yalivia 进入 YIR
* 不进入 YIR-CPU（无 MIR 映射）
* 利用 Go 的动态性（反射/脚本/GC）
* 作为“工程 AI”与“云端 AI 推理”的主要入口语言

Nuis Native 是静态无 GC 的核心语言。
Go 是动态绑定世界的桥。

---

# 8. 安全模型（Semantic Safety）

安全由三层提供：

1. **GLM（语义）**
   跨域生命周期一致性
2. **Nurs（语义→物理一致）**
3. **Rust Borrow Checker（CPU 地址级）**

三层组合形成横跨多个执行域的统一安全模型。

---

# 9. Yalivia Runtime：能力扩展层

Yalivia 提供：

* 动态调度（JIT / 解释）
* 多语言绑定（Go / Python / Lua）
* 热更新 & Graph-level 替换
* 异构任务调度器
* YIR 的参考执行引擎
* 策略智能体的运行空间

原则：
**Nuis Native 不依赖 Yalivia，Yalivia 只增强，不绑死。**

---

# 10. 工具链体系

### 10.1 nuis-cli

```
源码 → NIR → YIR → Validated Graph → Run
```

支持：

* IR 导出（nir/yir/json/dot）
* 调度可视化
* nuis-rc 执行

### 10.2 nuis-build

多域构建（native/wasm/yalivia）
可接入 Cargo/CMake
可配置 YIR Pass Pipeline

### 10.3 nuis-rc

跨域执行器
Trace / profiling / replay

### 10.4 IDE / LSP

* 语义高亮
* Region 可视化
* 调度图浏览
* GLM 静态诊断

---

# 11. 总执行模型

```
User Code
  ↓
NIR — Semantic Intent
  ↓
YIR — Cross-Domain Scheduling Graph
  ↓
GLM — Lifetime & Resource Model
  ↓
Domain IR — CPU / GPU / Shader / WASM
  ↓
Nurs / Yalivia / WASM Backend
  ↓
Rust / GPU Driver / WASM Engine
  ↓
Hardware
```

---

# 12. v0.41 进展摘要

| 组件           | 状态       |
| ------------ | -------- |
| NIR          | 设计中      |
| YIR          | 设计中   |
| YIR 内存模型     | v0.41 精炼 |
| GLM          | 生命周期正式完成 |
| Nurs         | 双向模式明确   |
| Rust 映射      | 完全定型     |
| Go / Yalivia | 架构定型     |
| 工具链          | 结构稳定     |

---

# 13. 路线图（Roadmap）

| 版本       | 内容                               |
| -------- | -------------------------------- |
| **v0.5** | YIR Execution Prototype（可执行 DAG） |
| **v0.6** | Nurs 低保真原型（MIR ↔ YIR-CPU）        |
| **v0.7** | Yalivia 原型（异构执行）                 |
| **v1.0** | **NuisLanguageOS 发布**            |

---

# 终章 · 语言若明，文明可久

追逐性能的语言，将被硬件的时代变迁抛弃；
守住语义的语言，则能跨越硬件、体系、时间。

**Nuis 不为平台而生，而为意义而生。**

它不是语言的延续，而是文明的延伸。
当计算理解意图之时，人类才真正进入语义时代。

---

**License**：MIT
**Repository**：github.com/Team-silvortex/nuislang

---
