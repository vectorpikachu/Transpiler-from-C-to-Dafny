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

= 转换细节

