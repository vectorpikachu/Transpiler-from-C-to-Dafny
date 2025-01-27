#import "@preview/cuti:0.3.0": show-cn-fakebold
#show: show-cn-fakebold

#let title = "A Transpiler from C to Dafny"
#let author = "VectorPikachu"
#let date =  datetime.today().display("[month repr:long] [day], [year]")
#set text(
  lang: "zh",
  font: ("Libertinus Serif", "SimSun"),
  region: "cn",
  size: 12pt,
)
#set page(
  "a4",
  margin: 1in,
  numbering: "1 / 1"
)
#show raw: set text(font: "Inconsolata", size: 12pt)
#set heading(
  numbering: "1.1.1.1."
)
#align(center,
[#block(text(size: 20pt, weight: "bold", title))
#block(text(size: 14pt, author))
#block(text(size: 12pt, date))])

#outline(
  title: "Contents",
  indent: 1.5em,
)

#pagebreak()

= Basic Usage

本项目使用 `Rust` 编写，使用 `cargo` 进行构建. 首先，确保你已经安装了 `Rust` 和 `cargo`. 

```bash
cargo run -- -i <input_file> -o <output_file>
```

= Translation Details

我为每个表达式手动标记了类型, 以试图捕捉 C 语言中的强制类型转换, 并将其转换为 Dafny 中的 `as` 操作符. 我们可以在打印的时候加上这些操作符.

= Soundness Proof

假设我们有这样几个转换:
- $T_p$: 从C的程序片段到Dafny的程序片段
- $T_c$: 从C的规约到Dafny的规约

对于任意C的霍尔三元组 $angle.l c_1, p, c_2 angle.r$, 其对应的Dafny三元组为 $angle.l T_c (c_1), T_p (p), T_c (c_2) angle.r$.

Soundness定义为如果后者成立, 那么前者一定成立.

+ 现在假设 $p$ 中出现了 `int` 类型, 并且出现了 `int` 的溢出操作, 那么 $p$ 后面进行什么操作都是可以的, 因为这是一个UB, 所以 $c_2$ 无论如何都将成立, 所以我们可以直接把C语言中的 `int` 建模为Dafny中的 `int` 类型.
+ 如果 $p$ 中出现了 `unsigned char` 和 `unsigned int`, 他们的溢出操作是有定义的回绕, 所以必须是 `bv` 类型.

