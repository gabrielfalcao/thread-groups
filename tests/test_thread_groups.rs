use thread_groups::{Error, Result, ThreadGroup};

#[test]
fn test_join() -> Result<()> {
    let mut threads = ThreadGroup::<String>::with_id(format!("{}:{}", module_path!(), line!()));
    for number in 401..409 {
        threads.spawn(move || format!("{}", number))?;
    }
    let data = threads.join()?;
    assert_eq!(data, "401");
    assert!(threads.errors().is_empty());
    Ok(())
}

#[test]
fn test_results() -> Result<()> {
    let mut threads = ThreadGroup::<u32>::with_id(format!("{}:{}", module_path!(), line!()));
    for number in 401..409 {
        threads.spawn(move || {
            if number % 2 == 0 && number < 407 {
                panic!("synthetic error at number {}", number)
            }
            number
        })?;
    }
    let data = threads.results();
    let ok_data = data
        .iter()
        .filter(|result| result.is_ok())
        .map(|result| result.clone().unwrap())
        .collect::<Vec<u32>>();
    let err_data = data
        .iter()
        .filter(|result| result.is_err())
        .map(|result| result.clone().err().unwrap())
        .collect::<Vec<Error>>();

    assert_eq!(ok_data, vec![401, 403, 405, 407, 408]);
    assert_eq!(err_data.len(), 3);
    assert_eq!(threads.errors().len(), 3);
    Ok(())
}

#[test]
fn test_as_far_as_ok() -> Result<()> {
    let mut threads = ThreadGroup::<u32>::with_id(format!("{}:{}", module_path!(), line!()));
    for number in 401..409 {
        threads.spawn(move || {
            if number % 2 == 0 && number < 407 {
                panic!("synthetic error at number {}", number)
            }
            number
        })?;
    }
    let data = threads.as_far_as_ok();
    assert_eq!(data, vec![401, 403, 405, 407, 408]);
    assert_eq!(threads.errors().len(), 3);
    Ok(())
}

#[test]
fn test_all_ok() -> Result<()> {
    let mut threads = ThreadGroup::<String>::with_id(format!("{}:{}", module_path!(), line!()));
    for number in 401..409 {
        threads.spawn(move || format!("{}", number))?;
    }
    let mut data = threads.all_ok()?;
    data.sort();

    assert_eq!(
        data,
        vec!["401", "402", "403", "404", "405", "406", "407", "408"]
    );
    assert!(threads.errors().is_empty());
    Ok(())
}
