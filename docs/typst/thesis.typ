#set page(
  paper: "a4",
  margin: (
    left: 30mm,
    right: 15mm,
    top: 20mm,
    bottom: 20mm,
  ),
)

#set text(
  lang: "ru",
  font: "Times New Roman",
  size: 14pt,
)

#set par(
  justify: true,
  first-line-indent: 12.5mm,
  leading: 0.65em,
)

#show quote: set align(right)
#show quote: set par(first-line-indent: 0pt, justify: false)
#show heading.where(level: 1): set align(center)
#show heading.where(level: 1): set block(above: 0pt, below: 1em)
#show heading.where(level: 1): set text(size: 14pt)

#show heading.where(level: 2): set align(center)
#show heading.where(level: 2): set block(above: 1em, below: 0.6em)
#show heading.where(level: 2): set text(size: 14pt)

#include "thesis-content.typo"
