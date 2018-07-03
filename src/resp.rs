use std::io::BufRead;

use failure::Error;
use httparse;

pub struct Response {
    pub code: u16,
    pub message: String,
    pub headers: Vec<(String, String)>,
}

impl Response {
    pub fn get_header<S: AsRef<str>>(&self, name: S) -> Option<&str> {
        let name = name.as_ref();
        for (k, v) in &self.headers {
            if name.eq_ignore_ascii_case(k) {
                return Some(v);
            }
        }

        None
    }

    pub fn content_length(&self) -> Option<u64> {
        self.get_header("Content-Length")
            .and_then(|h| h.parse().ok())
    }

    pub fn is_ok(&self) -> bool {
        200 == self.code
    }
}

pub fn read_until_empty_line<R: BufRead>(mut from: R) -> Result<(Vec<u8>, usize), Error> {
    let mut ret = Vec::with_capacity(256);
    let mut lines = 0;
    loop {
        from.read_until(b'\n', &mut ret)?;
        lines += 1;
        assert_eq!(ret[ret.len() - 1], b'\n');
        if (ret.len() >= 3 && b"\n\r" == &ret[ret.len() - 3..ret.len() - 1])
            || ret.len() >= 2 && b'\n' == ret[ret.len() - 2]
        {
            break;
        }
    }

    Ok((ret, lines))
}

pub fn read_response<R: BufRead>(from: R) -> Result<Response, Error> {
    let (data, lines) = read_until_empty_line(from)?;
    let mut header_records = vec![httparse::EMPTY_HEADER; lines];
    let mut resp = httparse::Response::new(&mut header_records);

    resp.parse(&data)?;
    Ok(Response {
        code: resp.code.ok_or_else(|| format_err!("no code"))?,
        message: resp
            .reason
            .ok_or_else(|| format_err!("no reason"))?
            .to_string(),
        headers: resp
            .headers
            .iter()
            .flat_map(|h| String::from_utf8(h.value.to_vec()).map(|v| (h.name.to_string(), v)))
            .collect(),
    })
}
