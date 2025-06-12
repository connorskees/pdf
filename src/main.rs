use std::{borrow::Cow, rc::Rc};

use pdf::{ContentLexer, Parser, PdfResult, Renderer};

fn main() -> PdfResult<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(String::new);
    let page = args.next().map(|n| n.parse::<u32>().unwrap()).unwrap_or(1);
    let mut parser = Parser::new(path)?;

    // let mut content = ContentLexer::new(Cow::Borrowed(
    //     b"
    //     .23999999 0 0 -.23999999 0 792 cm
    //     q
    //     116.666664 116.666664 2316.6665 3066.6665 re
    //     W* n
    //     q
    //     1.6953125 0 0 1.6953125 116.666664 116.666664 cm
    //     0 0 0 RG 0 0 0 rg
    //     /G3 gs
    //     0 0 1368 1810 re
    //     f
    //     0 0 1368 1128 re
    //     f
    //     0 0 0 RG 0 0 0 rg
    //     0 72 1368 110 re
    //     f
    //     Q
    //     Q
    //     q
    //     116.666664 343.83853 2316.6665 81.375 re
    //     W* n
    //     q
    //     1.6953125 0 0 1.6953125 116.666664 114.971352 cm
    //     0 0 0 RG 0 0 0 rg
    //     /G3 gs
    //     0 183 m
    //     0 134 l
    //     1368 134 l
    //     1368 183 l
    //     0 183 l
    //     h
    // ",
    // ));
    // let mut content = ContentLexer::new(Cow::Borrowed(
    //     b"
    //     .23999999 0 0 -.23999999 0 792 cm
    //     q
    //     % 116.666664 116.666664 2316.6665 3066.6665 re
    //     % W* n
    //     % q
    //     1.6953125 0 0 1.6953125 116.666664 116.666664 cm
    //     .9882 .9804 .9647 RG .9882 .9804 .9647 rg
    //     /G3 gs
    //     0 0 1368 5430 re
    //     f
    //     .9961 .8314 .2471 RG .9961 .8314 .2471 rg
    //     0 0 1368 1810 re
    //     f
    // ",
    // ));

    for page in parser.pages().into_iter().skip(page as usize - 1) {
        let mut content = parser.page_contents(&page).unwrap();

        let renderer = Renderer::new(&mut content, &mut parser.lexer, Rc::clone(&page));

        renderer.render().unwrap();
        break;
    }

    Ok(())
}
