DFIR Algebra (v0.01)

1 Primitive Set

DFIR 由固定原语集合构成：

MoveValue
CopyWindow
ImmutableWindow
PhantomMarker
InputPipe
OutputPipe
ResourceHandleTable

原语集合 不可扩展。

扩展必须通过：

primitive composition

实现。  ￼

⸻

2 Type Domain

DFIR 类型空间定义为：

Value<T>
Window<T>
Handle<R>
Marker<Tag>
Pipe<T>

其中：

T ∈ PrimitiveValue

PrimitiveValue：

int
float
struct


⸻

3 Pipe System

Pipe 表示静态数据流边：

Pipe<T>

合法类型：

Pipe<Value<T>>
Pipe<Window<T>>
Pipe<Handle<R>>
Pipe<Marker<Tag>>

Pipe 不允许嵌套：

Pipe<Pipe<T>> ❌

Pipe 必须满足：

OutputPipe<T> → InputPipe<T>

类型必须完全匹配。

⸻

4 Window Algebra

Window 表示数据视图：

Window =
    base
    offset
    shape
    stride

Window 允许递归嵌套：

Window<Value<T>>
Window<Window<Value<T>>>
Window<Window<Window<T>>>

禁止：

Window<Handle<R>>
Window<Marker<Tag>>

原因：

Window = memory view

Handle 与 Marker 不属于 memory。

⸻

5 Value Algebra

Value 表示标量数据：

Value<int>
Value<float>
Value<struct>

struct 仅允许包含：

Value

禁止包含：

Window
Handle
Marker
Pipe

这样可以保证：

layout determinism


⸻

6 Marker Algebra

Marker 为零尺寸语义 token：

Marker<Tag>

用途：

logical boundary
lifetime barrier
pipeline stage marker
debug trace marker

Marker 不携带数据。

允许链式组合：

Pipe
  ↓
Marker<A>
  ↓
Marker<B>
  ↓
Marker<C>

形成控制信号链。

⸻

7 Primitive × Type Compatibility Matrix

Primitive	Value	Window	Handle	Marker
MoveValue	✓	✗	✗	✗
CopyWindow	✗	✓	✗	✗
ImmutableWindow	✗	✓	✗	✗
PhantomMarker	✗	✗	✗	✓
Pipe	✓	✓	✓	✓

这张矩阵是：

DFIR verifier core rule


⸻

8 Resource Handle Semantics

Handle 表示资源引用：

Handle<Resource>

例如：

Handle<CPUCore>
Handle<GPUDevice>
Handle<AcceleratorUnit>

Handle 只能存在于：

Pipe<Handle<R>>

禁止：

Window<Handle<R>>
MoveValue<Handle<R>>

Handle 表示：

indirection reference

而不是数据。

⸻

9 Dataflow Legality Rules

Verifier 必须保证：

Type Compatibility

Pipe<T> endpoints match

Ownership Flow

MoveValue transfers ownership
source invalidated

Window Validity

offset + shape within bounds
stride legal

Marker Placement

marker cannot produce data
marker cannot own resources

Graph Connectivity

pipe endpoints valid
no disconnected data edges


⸻

10 DFIR Composition Principle

DFIR 的表达能力来自：

primitive composition

而不是：

primitive expansion

复杂行为通过组合实现，例如：

Window tiling
stream pipelines
fabric synchronization
resource fencing


⸻

11 Canonical Execution Model

DFIR 表达为：

typed pipe network

执行模型：

OutputPipe<T>
      ↓
Primitive
      ↓
Primitive
      ↓
InputPipe<T>

该模型可被 lowering 为：

fabric worker pipeline

或未来硬件数据通道。

⸻

12 Algebra Summary

DFIR 可以抽象为：

DFIR =
Primitive
×
Type
×
PipeGraph

其中：

Primitive = fixed set
Type = composable
PipeGraph = static topology


⸻

13 Relation to Execution IR

完整程序表示为：

Program =
ExecutionGraph (YIR)
×
DataFabricGraph (DFIR)

YIR 描述：

compute
sync
resource usage

DFIR 描述：

data movement
views
control markers

两者正交。  ￼
