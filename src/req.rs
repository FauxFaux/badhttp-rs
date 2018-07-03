use std::borrow::Borrow;
use std::fmt;
use std::io::Write;

use failure::Error;
use url;
use url::Host;
use url::Url;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum Method {
    GET,
    POST,
    HEAD,
    PUT,
    DELETE,
    OPTIONS,
    PATCH,
    CONNECT,
    TRACE,
}

pub trait IntoUrl {
    fn into_url(self) -> Result<Url, url::ParseError>;
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }
}

impl<'u> IntoUrl for &'u Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self.clone())
    }
}

impl<'s> IntoUrl for &'s str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        self.parse()
    }
}

impl<'s> IntoUrl for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        self.parse()
    }
}

pub fn write_raw<'s, W, M, U, H, HK, HV>(
    mut into: W,
    method: M,
    uri: U,
    http_one_one: bool,
    headers: H,
) -> Result<(), Error>
where
    W: Write,
    M: fmt::Display,
    U: fmt::Display,
    H: IntoIterator,
    H::Item: Borrow<(HK, HV)>,
    HK: fmt::Display,
    HV: fmt::Display,
{
    write!(
        into,
        "{} {} HTTP/1.{}\r\n",
        method,
        uri,
        if http_one_one { "1" } else { "0" }
    )?;

    for header in headers {
        let (name, value) = header.borrow();
        write!(into, "{}: {}\r\n", name, value)?;
    }

    write!(into, "\r\n")?;

    Ok(())
}

pub fn write_body_plus_boring_headers<W, M, U, H, B>(
    mut into: W,
    method: M,
    url: U,
    headers: H,
    body: B,
) -> Result<(), Error>
where
    W: Write,
    M: fmt::Display,
    U: IntoUrl,
    H: IntoIterator<Item = (String, String)>,
    B: AsRef<[u8]>,
{
    let url = url.into_url()?;

    let uri = if let Some(query) = url.query() {
        format!("{}?{}", url.path(), query)
    } else {
        url.path().to_string()
    };

    let body = body.as_ref();

    let mut extra_headers = Vec::with_capacity(2);
    extra_headers.push(("Content-Length".to_string(), format!("{}", body.len())));

    if let Some(Host::Domain(host)) = url.host() {
        extra_headers.push(("Host".to_string(), host.to_string()));
    }

    write_raw(
        &mut into,
        method,
        uri,
        true,
        headers.into_iter().chain(extra_headers),
    )?;

    into.write_all(body)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Method;

    #[test]
    fn write_ergonomics() {
        use super::write_raw;
        let mut v = Vec::new();
        write_raw(
            &mut v,
            Method::GET,
            "/test",
            true,
            &[("Host", "fau.xxx"), ("X-Foo", "Bar")],
        ).unwrap();
        assert_eq!(
            String::from_utf8(v).unwrap(),
            "GET /test HTTP/1.1\r\nHost: fau.xxx\r\nX-Foo: Bar\r\n\r\n"
        )
    }
}
