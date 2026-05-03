#import "@preview/mmdr:0.2.1": mermaid

#let mermaid_diagram(code, caption) = [
  #align(center)[#mermaid(code)]
  #v(2mm)
  #align(center)[#caption]
  #v(4mm)
]
