# PDF parser and renderer

Early stage, focused only on well-formed input and Type 1 fonts

## Reading

### Specifications and formats

https://www.adobe.com/jp/print/postscript/pdfs/PLRM.pdf<br>
https://www.adobe.com/content/dam/acom/en/devnet/pdf/pdfs/PDF32000_2008.pdf<br>
https://adobe-type-tools.github.io/font-tech-notes/pdfs/T1_SPEC.pdf<br>
http://www.libpng.org/pub/png/spec/1.2/png-1.2.pdf<br>
https://www.adobe.com/content/dam/acom/en/devnet/postscript/pdfs/TN5603.Filters.pdf<br>
https://www.adobe.com/content/dam/acom/en/devnet/actionscript/articles/psrefman.pdf<br>
https://developer.apple.com/fonts/TrueType-Reference-Manual<br>
https://hepunx.rl.ac.uk/~adye/psdocs/ref/REF.html<br>
http://fileformats.archiveteam.org/wiki/PostScript_binary_object_format<br>

### Algorithms

http://members.chello.at/~easyfilter/Bresenham.pdf<br>
https://en.wikipedia.org/wiki/Ascii85<br>
https://pomax.github.io/bezierinfo<br>
https://secure.math.ubc.ca/~cass/piscript/type1.pdf<br>
https://ltr.wtf/explained/bidiintro.html<br>

### Other resources

https://speakerdeck.com/ange/lets-write-a-pdf-file<br>
https://web.archive.org/web/20150110042057/http://home.comcast.net/~jk05/presentations/PDFTutorials.html/<br>
http://formats.kaitai.io/ttf/ttf.svg/<br>

## Non-features and spec deviations

This library will _never_ support network requests, database connections, or JavaScript execution.

In addition, features like file IO and Flash will only ever be disabled by default.

https://web-in-security.blogspot.com/2021/01/insecure-features-in-pdfs.html
