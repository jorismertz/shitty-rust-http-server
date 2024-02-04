pub mod response {
    use std::fmt::{format, Display};
    use std::str::FromStr;
    #[derive(Debug)]
    pub enum Status {
        Ok,
        Created,
        Accepted,
        BadRequest,
        NotFound,
        InternalServerError,
    }

    pub type Headers = Vec<(String, String)>;

    // pub trait HeaderFunctions {
    //     fn get_header(&self, name: &str) -> Option<(String, String)>;
    // }

    // impl HeaderFunctions for Headers {
    //     fn get_header(&self, name: &str) -> Option<(String, String)> {
    //         for (key, value) in self.into_iter() {
    //             if key == name {
    //                 return Some((key.to_string(), value.to_string()));
    //             }
    //         }
    //         return None;
    //     }
    // }

    impl Status {
        pub fn status_code(&self) -> i32 {
            return match self {
                Self::Ok => 200,
                Self::Created => 201,
                Self::Accepted => 202,
                Self::BadRequest => 400,
                Self::NotFound => 404,
                Self::InternalServerError => 500,
            };
        }
        pub fn response_string(&self) -> String {
            return format!("HTTP/1.1 {} {}", self.status_code(), self);
        }
    }

    #[derive(Debug)]
    pub enum Header {
        ContentType,
        ContentLength,
        Accept,
        AcceptCharset,
        AcceptEncoding,
        AcceptLanguage,
        Authorization,
        CacheControl,
        Connection,
        Cookie,
        Host,
        Referer,
        Server,
        Date,
    }

    impl Header {
        pub fn to_str(&self) -> String {
            let mut string = format!("{:?}", self);
            for (index, char) in string.clone().chars().into_iter().enumerate() {
                if char.is_uppercase() && index != 0 {
                    string.insert_str(index, "-");
                }
            }
            return string;
        }

        pub fn new(&self, value: &str) -> (String, String) {
            return (self.to_str().into(), value.into());
        }
    }

    pub struct Message<'a> {
        status: &'a Status,
        body: Option<&'a str>,
        headers: &'a Headers,
    }

    impl<'a> Message<'a> {
        pub fn new(status: &'a Status, body: Option<&'a str>, headers: &'a Headers) -> Message<'a> {
            return Message {
                body,
                headers,
                status,
            };
        }
    }

    impl<'a> ToString for Message<'a> {
        fn to_string(&self) -> String {
            let mut response: Vec<String> = Vec::new();

            response.push(format!(
                "HTTP/1.1 {} {} \r\n",
                self.status.status_code(),
                self.status
            ));

            for (key, value) in self.headers {
                response.push(format!("{}: {}\r\n", key, value));
            }

            if let Some(body) = self.body {
                response.push(format!("\r\n{}", body));
            }

            let joined_response_message = response.join("");
            return joined_response_message;
        }
    }

    impl Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let as_string = format!("{:?}", self);

            let mut result: Vec<char> = Vec::new();

            for (i, char) in as_string.chars().enumerate() {
                if char.is_uppercase() && i != 0 {
                    result.push(char::from_str(" ").unwrap());
                    result.push(char);
                    continue;
                }
                result.push(char)
            }

            let as_string: String = result.iter().collect();

            write!(f, "{}", as_string.to_uppercase()).expect("it to write yk");
            return Ok(());
        }
    }
}
