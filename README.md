# PDF parser and renderer

An attempt at writing a PDF renderer from as close to scratch as possible.

This project includes from-scratch parsers/interpreters/renderers/implementations of PostScript, ICC profiles, PNG compression, Type 1 fonts (.pfb files), TrueType fonts (.ttf files), CFF (compact fonts), 2d path manipulation and rasterization, among others. 

This library is intended to crash on malformed input and makes heavy use of assertions to ensure that parsing is never silently incorrect. 

This library is not suited for the regular user, and has limited utility outside of the niche cases that I personally use it for.

As a parser, this projects is the most comprehensive pure-rust implementation I am aware of, though as a renderer it is less complete.

It should be noted that this library will intentionally deviate from the spec when there is the potential for network/disk access or RCE through PDF files. See https://web-in-security.blogspot.com/2021/01/insecure-features-in-pdfs.html for a good reference of just some of such features.

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
https://adobe-type-tools.github.io/font-tech-notes/pdfs/5176.CFF.pdf<br>
https://adobe-type-tools.github.io/font-tech-notes/pdfs/5177.Type2.pdf<br>
https://www.color.org/icc1v42.pdf<br>
https://adobe-type-tools.github.io/font-tech-notes/pdfs/5049.StemSnap.pdf<br>
https://www.pdfa.org/norm-refs/5620.PortableJobTicket.pdf<br>
https://www.itu.int/rec/T-REC-T.6-198811-I/en<br>

##### True Type Resources

https://www.truetype-typography.com/tthints.htm<br>
https://learn.microsoft.com/en-us/typography/opentype/spec/ttch01<br>
https://developer.apple.com/fonts/TrueType-Reference-Manual<br>
https://xgridfit.sourceforge.net/round.html<br>

### Algorithms and Relevant Literature

http://members.chello.at/~easyfilter/Bresenham.pdf<br>
https://en.wikipedia.org/wiki/Ascii85<br>
https://pomax.github.io/bezierinfo<br>
https://secure.math.ubc.ca/~cass/piscript/type1.pdf<br>
https://ltr.wtf/explained/bidiintro.html<br>
https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=1000&context=facpub<br>
https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html<br>
http://www.cccg.ca/proceedings/2004/36.pdf<br>

### Other resources

https://speakerdeck.com/ange/lets-write-a-pdf-file<br>
https://web.archive.org/web/20150110042057/http://home.comcast.net/~jk05/presentations/PDFTutorials.html/<br>
http://formats.kaitai.io/ttf/ttf.svg/<br>
https://www.tinaja.com/glib/interdic.pdf<br>
https://personal.math.ubc.ca/~cass/piscript/type1.pdf<br>
