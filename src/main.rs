use std::rc::Rc;

use pdf::{Parser, PdfResult, Renderer};

fn main() -> PdfResult<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(String::new);
    let page = args.next().map(|n| n.parse::<u32>().unwrap()).unwrap_or(1);
    let mut parser = Parser::new(path)?;

    for page in parser.pages().into_iter().skip(page as usize - 1) {
        let mut content = parser.page_contents(&page).unwrap();

        let renderer = Renderer::new(&mut content, &mut parser.lexer, Rc::clone(&page));

        renderer.render().unwrap();
        break;
    }

    Ok(())
}
