#import "@preview/cuti:0.3.0": show-cn-fakebold
#import "@preview/codly:1.2.0": *
#import "@preview/codly-languages:0.1.1": *
#import "@preview/tablem:0.1.0": tablem
#import "@preview/tablex:0.0.9": tablex
#show: codly-init.with()
#show: show-cn-fakebold

#let title = "A Transpiler from C to Dafny"
#let author = "VectorPikachu"
#let date =  datetime.today().display("[month repr:long] [day], [year]")
#set text(
  lang: "en",
  font: ("Libertinus Serif", "SimSun"),
  region: "cn",
  size: 12pt,
)
#set page(
  "a4",
  margin: 1in,
  numbering: "1 / 1"
)
#set raw(syntaxes: ("./Dafny.sublime-syntax"))
#show raw: set text(font: "Inconsolata", size: 12pt)



#set heading(
  numbering: "1.1.1.1."
)
#show link: it => text(
  fill: rgb("#0645AD"),
)[#it]
#align(center,
[#block(text(size: 20pt, weight: "bold", title))
#block(text(size: 14pt, author))
#block(text(size: 12pt, date))])

#align(center, image("./assets/Transpiler-from-C-to-Dafny.svg", width: 80%))

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

我提供了一个 ```rust traverse_tree()```, 它可以遍历语法树并打印出每个节点的类型. 我们可以通过这个函数来查看语法树的结构.

- `preproc_include` 类型是头文件
- `preproc_function_def` 类型是宏中的函数定义. e.g. `preproc_function_def: #define assume(e) if(!(e)) exit(-1);`
- `preproc_def` 类型是宏定义. e.g. `preproc_def: #define a (2)`
- `function_definition` 类型是函数定义. e.g. `function_definition: int main() { return 0; }`

我为每个表达式手动标记了类型, 以试图捕捉 C 语言中的强制类型转换, 并将其转换为 Dafny 中的 `as` 操作符. 我们可以在打印的时候加上这些操作符.

== Arrays

关于数组: `seq<T>` 是 Dafny 中一个更加基本的类型, 而且虽然没有 `old` 操作符的支持, 但是我们可以通过下面的形式来模拟:
```Dafny
ghost var prevElements := a[..];
while // ...
  invariant a[lo..hi] == prevElements[lo..hi]
{
  // ...
}

// The above is equivalent to:
while // ...
  invariant a[lo..hi] == old(a[lo..hi])
{
  // ...
}
```

但是 C 语言代码需要 update 一个数组, 这方面来看还是 `array<T>` 更合适, 所以我们选择使用 `array<T>`.

= Soundness Proof

假设我们有这样几个转换:
- $T_p$: 从C的程序片段到Dafny的程序片段
- $T_c$: 从C的规约到Dafny的规约

对于任意C的霍尔三元组 $angle.l c_1, p, c_2 angle.r$, 其对应的Dafny三元组为 $angle.l T_c (c_1), T_p (p), T_c (c_2) angle.r$.

Soundness定义为如果后者成立, 那么前者一定成立.

+ 现在假设 $p$ 中出现了 `int` 类型, 并且出现了 `int` 的溢出操作, 那么 $p$ 后面进行什么操作都是可以的, 因为这是一个UB, 所以 $c_2$ 无论如何都将成立, 所以我们可以直接把C语言中的 `int` 建模为Dafny中的 `int` 类型.
+ 如果 $p$ 中出现了 `unsigned char` 和 `unsigned int`, 他们的溢出操作是有定义的回绕, 所以必须是 `bv` 类型.

#bibliography(
  "./ref.bib",
  style: "ieee",
  full: true,
)