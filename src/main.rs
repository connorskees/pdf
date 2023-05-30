use std::rc::Rc;

use pdf::{Parser, PdfResult, Renderer};

fn main() -> PdfResult<()> {
    let mut parser = Parser::new("corpus/Christopher Smith Resume.pdf")?;

    for page in parser.pages().into_iter().skip(0) {
        let mut content = parser.page_contents(&page).unwrap();

        let renderer = Renderer::new(&mut content, &mut parser.lexer, Rc::clone(&page));

        renderer.render().unwrap();
        break;
    }

    // dbg!(parser.run().unwrap());

    Ok(())
}
