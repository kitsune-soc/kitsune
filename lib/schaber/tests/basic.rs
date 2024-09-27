use schaber::Scraper;

#[test]
fn select_link() {
    let html = r#"
        <div id="hello">
            <a href="http://druckbrudi.lab">
                PRINT MORE BLÅHAJ CATEARS!
            </a>
        </div>
    "#;

    let mut link_url = None;
    let scraper = Scraper::new("a").unwrap();

    scraper
        .process(html, |element| {
            link_url = element.get_attribute("href");
        })
        .unwrap();

    assert_eq!(link_url.as_deref(), Some("http://druckbrudi.lab"));
}
