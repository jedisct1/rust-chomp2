use benchmark_simple::*;
use chomp1::prelude::*;
use chomp1::{__parse_internal, __parse_internal_or, parse, parser};

macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

#[derive(Debug)]
struct Request<B> {
    method: B,
    uri: B,
    version: B,
}

#[derive(Debug)]
struct Header<B> {
    name: B,
    value: Vec<B>,
}

fn is_token(c: u8) -> bool {
    match c {
        128..=255 => false,
        0..=31 => false,
        b'(' => false,
        b')' => false,
        b'<' => false,
        b'>' => false,
        b'@' => false,
        b',' => false,
        b';' => false,
        b':' => false,
        b'\\' => false,
        b'"' => false,
        b'/' => false,
        b'[' => false,
        b']' => false,
        b'?' => false,
        b'=' => false,
        b'{' => false,
        b'}' => false,
        b' ' => false,
        _ => true,
    }
}

fn is_horizontal_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}
fn is_space(c: u8) -> bool {
    c == b' '
}
fn is_not_space(c: u8) -> bool {
    c != b' '
}
fn is_end_of_line(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}
fn is_http_version(c: u8) -> bool {
    c.is_ascii_digit() || c == b'.'
}

fn end_of_line<I: U8Input>(i: I) -> SimpleResult<I, u8> {
    parse! {i; (token(b'\r') <|> ret b'\0') >> token(b'\n')}
}

fn http_version<I: U8Input>(i: I) -> SimpleResult<I, I::Buffer> {
    parse! {i;
        string(b"HTTP/");
        take_while1(is_http_version)
    }
}

fn request_line<I: U8Input>(i: I) -> SimpleResult<I, Request<I::Buffer>> {
    parse! {i;
        let method  = take_while1(is_token);
                      take_while1(is_space);
        let uri     = take_while1(is_not_space);
                      take_while1(is_space);
        let version = http_version();

        ret Request {
            method,
            uri,
            version,
        }
    }
}

fn message_header_line<I: U8Input>(i: I) -> SimpleResult<I, I::Buffer> {
    parse! {i;
                   take_while1(is_horizontal_space);
        let line = take_till(is_end_of_line);
                   end_of_line();

        ret line
    }
}

fn message_header<I: U8Input>(i: I) -> SimpleResult<I, Header<I::Buffer>> {
    parse! {i;
        let name  = take_while1(is_token);
                    token(b':');
        let lines = many1(message_header_line);

        ret Header {
            name,
            value: lines,
        }
    }
}

#[inline(never)]
fn request<I: U8Input>(i: I) -> SimpleResult<I, (Request<I::Buffer>, Vec<Header<I::Buffer>>)> {
    parse! {i;
        let r = request_line();
                end_of_line();
        let h = many(message_header);
                end_of_line();

        ret (r, h)
    }
}

pub fn bench() {
    single_request();
    single_request_large();
    single_request_minimal();
    multiple_requests();
}

fn single_request() {
    let data = b"GET / HTTP/1.1\r
Host: www.reddit.com\r
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.8; rv:15.0) Gecko/20100101 Firefox/15.0.1\r
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r
Accept-Language: en-us,en;q=0.5\r
Accept-Encoding: gzip, deflate\r
Connection: keep-alive\r
\r
\r";

    let res = Bench::new().run(&Options::default(), || parse_only(request, data));
    println!("{}: {}", function_name!(), res);
}

fn single_request_minimal() {
    let data = b"GET / HTTP/1.1\r
Host: localhost\r
\r
\r";

    let res = Bench::new().run(&Options::default(), || parse_only(request, data));
    println!("{}: {}", function_name!(), res);
}

fn single_request_large() {
    let data = b"GET /i.gif?e=eyJhdiI6NjIzNTcsImF0Ijo1LCJjbSI6MTE2MzUxLCJjaCI6Nzk4NCwiY3IiOjMzNzAxNSwiZGkiOiI4NmI2Y2UzYWM5NDM0MjhkOTk2ZTg4MjYwZDE5ZTE1YyIsImRtIjoxLCJmYyI6NDE2MTI4LCJmbCI6MjEwNDY0LCJrdyI6Ii1yZWRkaXQuY29tIiwibWsiOiItcmVkZGl0LmNvbSIsIm53Ijo1MTQ2LCJwYyI6MCwicHIiOjIwMzYyLCJydCI6MSwicmYiOiJodHRwOi8vd3d3LnJlZGRpdC5jb20vIiwic3QiOjI0OTUwLCJ1ayI6InVlMS01ZWIwOGFlZWQ5YTc0MDFjOTE5NWNiOTMzZWI3Yzk2NiIsInRzIjoxNDAwODYyNTkzNjQ1fQ&s=lwlbFf2Uywt7zVBFRj_qXXu7msY HTTP/1.1\r
Host: engine.adzerk.net\r
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.8; rv:15.0) Gecko/20100101 Firefox/15.0.1\r
Accept: image/png,image/*;q=0.8,*/*;q=0.5\r
Accept-Language: en-us,en;q=0.5\r
Accept-Encoding: gzip, deflate\r
Connection: keep-alive\r
Referer: http://static.adzerk.net/reddit/ads.html?sr=-reddit.com&bust2\r
Cookie: azk=ue1-5eb08aeed9a7401c9195cb933eb7c966\r
\r
\r";

    let res = Bench::new().run(&Options::default(), || parse_only(request, data));
    println!("{}: {}", function_name!(), res);
}

fn multiple_requests() {
    let data = include_bytes!("./data/http-requests.txt");

    let res = Bench::new().run(&Options::default(), || {
        let r: Result<Vec<_>, _> = parse_only(parser! {many(request)}, data);
        r
    });
    println!("{}: {}", function_name!(), res);
}
