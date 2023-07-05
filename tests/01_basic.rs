use convert_params::convert_args;

struct Orig {}

#[derive(Debug)]
struct Foo {}

impl TryFrom<Orig> for Foo {
    type Error = &'static str;
    fn try_from(_v: Orig) -> Result<Self, Self::Error> {
        Ok(Foo {})
    }
}

#[derive(Debug)]
struct Bar {}

impl TryFrom<Orig> for Bar {
    type Error = &'static str;
    fn try_from(_v: Orig) -> Result<Self, Self::Error> {
        Err("failed")
    }
}

#[derive(Debug, PartialEq)]
struct Error {
    s: &'static str,
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error { s }
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_err: std::convert::Infallible) -> Error {
        panic!("cannot happen")
    }
}

#[convert_args(_value1: Orig, _value2: &str)]
fn success(_i: u32, _value1: Foo, _value2: String) -> Result<(), Error> {
    Ok(())
}

#[convert_args(_value: Orig)]
fn failure(_value: Bar) -> Result<(), Error> {
    Ok(())
}

fn main() {
    assert!(success(42, Orig {}, "test").is_ok());
    assert_eq!(failure(Orig {}), Err(Error { s: "failed" }));
}
