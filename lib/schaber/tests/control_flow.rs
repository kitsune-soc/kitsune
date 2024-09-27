use schaber::Scraper;
use std::ops::ControlFlow;

#[test]
fn ends_after_break() {
    let html = r#"
        <div id="hello">
            <a href="http://druckbrudi.lab">
                PRINT MORE BLÅHAJ CATEARS!
            </a>

            <a href="http://evil.com">
                This link shall not be seen!
            </a>
        </div>
    "#;

    let mut link_url = None;
    let scraper = Scraper::new("a").unwrap();

    scraper
        .process(html, |element| {
            link_url = element.get_attribute("href");
            ControlFlow::Break(())
        })
        .unwrap();

    assert_eq!(link_url.as_deref(), Some("http://druckbrudi.lab"));
}

#[test]
fn continues_after_continue() {
    let html = r#"
        <div id="hello">
            <a href="http://druckbrudi.lab">
                PRINT MORE BLÅHAJ CATEARS!
            </a>

            <a href="https://good.org">
                This link shall be seen!
            </a>
        </div>
    "#;

    let mut link_url = None;
    let scraper = Scraper::new("a").unwrap();

    scraper
        .process(html, |element| {
            link_url = element.get_attribute("href");
            ControlFlow::Continue(())
        })
        .unwrap();

    assert_eq!(link_url.as_deref(), Some("https://good.org"));
}
