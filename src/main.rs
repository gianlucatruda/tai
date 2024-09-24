use reqwest;

/// Basic GET request to specified URL
fn do_get() -> Result<(), reqwest::Error> {
    let res = reqwest::blocking::get("http://httpbin.org/get")?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body: String = res.text()?;
    println!("Body:\n{}", body);

    Ok(())
}

fn main() {
    println!("TAI");
    let _ = do_get();
}
