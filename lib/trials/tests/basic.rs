use trials::attempt;

#[test]
fn works_declarative() {
    let result: Result<(), ()> = attempt! {
        Err(())?;
    };
    assert!(result.is_err());
}

#[futures_test::test]
async fn works_declarative_async() {
    let result: Result<(), ()> = attempt! { async
        async { Err(()) }.await?;
    };
    assert!(result.is_err());
}
