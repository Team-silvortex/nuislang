# 🪶 NuisLang

> **A Language Designed to Outlive Hardware**
> “语言的使命不是追随硬件，而是超越它。”

---

## 📚 Table of Contents

1. [Overview](#-overview)
2. [Philosophy](#-philosophy)
3. [Core Abstractions](#-core-abstractions)
4. [Compiler Architecture](#-compiler-architecture)
5. [Semantic Data Model](#-semantic-data-model)
6. [Type & Context System](#-type--context-system)
7. [Safety Model](#-safety-model)
8. [Runtime Layer](#-runtime-layer)
9. [Extensibility](#-extensibility)
10. [AI Strategy Engine](#-ai-strategy-engine)
11. [Semantic Debug API](#-semantic-debug-api)
12. [AI Semantic Advisor](#-ai-semantic-advisor)
13. [Evolution Roadmap](#-evolution-roadmap)
14. [Strategic Positioning](#-strategic-positioning)
15. [Final Words](#-final-words)

---

## 🌌 Overview

**NuisLang** 是一门以 **语义为中心（Semantics-Centric）** 的新型计算语言。
它旨在用稳定的语义抽象层统一 CPU、GPU、NPU、QPU 等多代架构，
并通过 AI 驱动的策略编译器，使“编译器”成为**可学习、可演化的智能体**。

> 🧭 不为平台编程，而为计算的意义编程。

---

## 🧠 Philosophy

| 层级      | 说明        | 稳定性  |
| ------- | --------- | ---- |
| **语义层** | 描述 “做什么”  | 永久稳定 |
| **策略层** | 决定 “怎么做”  | 可替换  |
| **执行层** | 实现 “在哪里做” | 可演化  |

**设计信条：**

1. 用户只描述意图，编译器决定实现。
2. 抽象必须有语义支撑，而非语法糖。
3. 时间兼容性 > 平台兼容性。
4. 编译器可学习、可自我重写。
5. 所有跨域通信、生命周期均可静态验证。

---

## ⚙️ Core Abstractions

### 1️⃣ `mod` — 类型化计算上下文

每个 `mod` 定义一个计算域（CPU / GPU / NPU / Quantum）。

```nuis
mod cpu Host { ... }
mod kernel Compute { ... }
mod shader Lighting { ... }
mod quantum QKernel { ... }
```

* `mod` 是独立语义空间；
* 同名类型可在不同 mod 下有独立实现；
* 模块间通信需通过 `channel` 显式声明。

---

### 2️⃣ `channel` — 跨域数据传输语义

```nuis
let ch = channel<Buffer<f32>, cpu::Host, kernel::Compute>::new();
ch.send(data);
kernel::Compute.dispatch(ch);
```

* 自动选择零拷贝、DMA、UVM 等策略；
* 生命周期静态验证；
* 支持 async / shared / stream 模式。

---

### 3️⃣ `strategy engine` — 策略智能编译层

```yaml
strategy:
  mode: auto
  model: ai_policy
```

* **Rule-driven**：基于规则的策略表。
* **AI-driven**：AI 模型预测编译策略。
* **Profile-driven**：性能反馈优化。

---

## 🧩 Compiler Architecture

```
Source (.ns)
  ↓
AST → Semantic IR (Nuis Dialect)
  ↓
Strategy Engine (Rules / AI)
  ↓
MLIR / LLVM Lowering
  ↓
SPIR-V / NVPTX / QIR / WASM
```

**特征：**

* Semantic IR：语义层中间表示；
* Strategy Engine：可学习优化层；
* Lowering Pass：自适应硬件架构。

---

## 🧱 Semantic Data Model (SDM)

三层语义结构：

| 层级                       | 职责             |
| ------------------------ | -------------- |
| **Mod Context Layer**    | 数据的语义作用域与内存边界  |
| **Entity Layer**         | 追踪对象的语义状态与生命周期 |
| **Lifetime Graph Layer** | 可视化数据的创建、迁移与析构 |

> 🧩 调试内存 → 看语义。
> Nuis Debug = Watch the Flow, not the Bytes.

---

## 🔣 Type & Context System

### 特征：

* 每个 `mod` 是执行上下文；
* 泛型是语义模板；
* 闭包捕获上下文；
* 同名类型跨上下文可独立实现；
* 多态是“语义多态”，非类型多态。

```nuis
mod physics(cpu) {
    struct Vec3 { x: f32, y: f32, z: f32 }
    impl Vec3 { fn len(&self) -> f32 { sqrt(x*x + y*y + z*z) } }
}

mod shader(shader) {
    struct Vec3 { x: f32, y: f32, z: f32 }
    impl Vec3 { fn len(&self) -> f32 { gpu_sqrt(x*x + y*y + z*z) } }
}
```

---

## 🧩 Safety Model

* 异构 Borrow Checker（扩展 Rust 所有权）；
* 每个 `Create` 对应唯一 `Drop`；
* 循环引用自动拒绝；
* 静态验证跨域生命周期与资源冲突。

---

## 🧰 Runtime Layer

运行时承担最小职责：

* 设备分配；
* Channel 同步；
* 策略模型加载；
* 性能采样与反馈。

语义与运行时解耦，允许完全替换。

---

## 🧬 Extensibility

通过插件系统注册新硬件域：

```nuis
domain plugin quantum {
    target = "qir"
    lowering_pass = "QuantumLoweringPass"
    resource_model = "QubitRegister"
}
```

自动获得：

```nuis
mod quantum QKernel { ... }
```

---

## 🤖 AI Strategy Engine

| 输入                             | 输出        |
| ------------------------------ | --------- |
| 模块拓扑 / Channel 特征 / Profile 数据 | 编译策略 JSON |

可反馈训练：
编译 → Profile → 反馈 → 模型微调。

> 编译器成为可学习体，而非静态规则机。

---

## 🧪 Semantic Debug API

| 方法                           | 功能       |
| ---------------------------- | -------- |
| `watch(entity)`              | 监视语义实体状态 |
| `trace_transfer(entity)`     | 跟踪迁移路径   |
| `lifemap(mod)`               | 生命周期图    |
| `assert_consistency()`       | 验证所有权一致性 |
| `snapshot(ctx)`              | 保存上下文快照  |
| `diff(snapshot1, snapshot2)` | 对比帧间差异   |

---

## 🧠 AI Semantic Advisor

AI 提供结构健康分析（非代码改写）：

| 分析维度     | 功能       |
| -------- | -------- |
| 跨 mod 调用 | 检查非法依赖   |
| impl 重复  | 建议抽象化或桥接 |
| 泛型过度     | 提醒合理约束   |
| 闭包上下文    | 提示性能优化   |
| 模块图复杂度   | 建议拆分或合并  |

---

## 🛠 Evolution Roadmap

| 版本   | 阶段目标                | 状态 |
| ---- | ------------------- | -- |
| v0.3 | 核心语义模型完成            | ✅  |
| v0.4 | LLVM / MLIR dialect | 🚧 |
| v0.5 | AI 策略引擎原型           | 🚧 |
| v0.6 | 多设备 runtime         | ⏳  |
| v0.7 | Quantum backend     | ⏳  |
| v1.0 | 稳定语义规范 + 工业编译栈      | 🏁 |

---

## 🪶 Strategic Positioning

| 对比项   | Mojo      | JAX    | **NuisLang** |
| ----- | --------- | ------ | ------------ |
| 核心理念  | 性能统一      | 数学抽象   | **语义统一**     |
| 抽象中心  | Tensor    | 函数图    | **计算意图**     |
| 扩展性   | Python 嵌入 | ML 图优化 | **语义层学习**    |
| AI 角色 | 加速推理      | 图优化    | **参与编译决策**   |
| 哲学高度  | 机器层       | 模型层    | **意义层**      |

---

## 🏔 Final Words

> Nuis 不为硬件编程，而为意义编程。
> 它不是语言的延续，而是语言的反思。
>
> 当编译器能理解“意图”，而不仅是“指令”，
> 那一刻，计算将第一次变得真正有思想。

---

### 💡 License & Vision

* License: MIT (to encourage open AI-compiler research)
* Maintainer: [Your Name or Alias]
* Repository: `github.com/Team-silvortex/nuislang.git`

