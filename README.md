# A Simple Transpiler from C to Dafny

使用 [tree-sitter](https://github.com/tree-sitter/tree-sitter) 和 [tree-sitter-c](https://github.com/tree-sitter/tree-sitter-c) 构建的 C to Dafny 转换器。

尽管 `assert(false)` 会报错，但是依然可以被解析。

## 转换方法

## Soundness

假设我们有这样几个转换:
- $T_p$: 从 C 的程序片段到 Dafny 的程序片段;
- $T_c$: 从 C 的规约到 Dafny 的规约.

对于任意C的霍尔三元组 $\langle c_1, p, c_2 \rangle$, 其对应的Dafny三元组为 $\langle T_c (c_1), T_p (p), T_c (c_2) \rangle$.

那么Soundness定义为如果后者成立, 那么前者一定成立.

+ 现在假设 $p$ 中出现了 `int` 类型, 并且出现了 `int` 的溢出操作, 那么 $p$ 后面进行什么操作都是可以的, 因为这是一个UB, 所以 $c_2$ 无论如何都将成立, 所以我们可以直接把 C 语言中的 `int` 建模为 Dafny 中的 `int` 类型.
+ 如果 $p$ 中出现了 `unsigned char` 和 `unsigned int`, 他们的溢出操作是有定义的回绕, 所以必须是 `bv` 类型.