use trials::attempt;

#[test]
#[cfg(feature = "proc-macro")]
fn does_catch() {
    trials::trials_stable! {
        fn is_odd(num: i32) -> bool {
            let result: Result<(), ()> = try {
                if num % 2 == 0 {
                    Err(())?;
                }
            };

            result.is_ok()
        }
    }

    assert!(is_odd(3));
    assert!(!is_odd(2));
}

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

#[test]
#[cfg(feature = "proc-macro")]
fn works_on_impl_block() {
    struct Test;

    #[trials::trials]
    impl Test {
        fn erroring() {
            let result: Result<(), ()> = try {
                let fallible_op = || Err(());
                fallible_op()?;
            };

            assert!(result.is_err());
        }
    }

    Test::erroring();
}

#[futures_test::test]
#[cfg(feature = "proc-macro")]
async fn does_catch_async() {
    #[trials::trials]
    async fn is_odd(num: i32) -> bool {
        let result: Result<(), ()> = try {
            if num % 2 == 0 {
                Err(())?;
            }
        };

        result.is_ok()
    }

    assert!(is_odd(3).await);
    assert!(!is_odd(2).await);
}
