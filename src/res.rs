pub mod http {
    use std::fmt::{format, Display};
    use std::str::FromStr;

    use serde::{Deserialize, Serialize, Serializer};
    #[derive(Debug)]
    pub enum Status {
        Ok,
        Created,
        Accepted,
        BadRequest,
        NotFound,
        InternalServerError,
    }

    // This serializer is used to unwrap the value of a Result type.
    // otherwise it will send back something like { field: { Ok: value } }
    fn serialize_result_string<T, E, S>(
        result: &Result<T, E>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: Display,
        E: Display,
        S: Serializer,
    {
        match result {
            Ok(value) => serializer.serialize_str(&value.to_string()),
            Err(error) => serializer.serialize_str(&error.to_string()),
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ResponseResult<T, E>
    where
        T: Display,
        E: Display,
    {
        pub ok: bool,
        #[serde(serialize_with = "serialize_result_string")]
        pub result: Result<T, E>,
    }

    pub type Headers = Vec<(String, String)>;

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
        AccessControlAllowOrigin,
    }

    impl Header {
        pub fn to_str(&self) -> String {
            let mut string = format!("{:?}", self);
            let mut chars: Vec<(usize, char)> = string.char_indices().into_iter().collect();
            chars.reverse();

            for (index, char) in &chars {
                if char.is_uppercase() && *index != 0 {
                    string.insert_str(*index, "-");
                }
            }

            return string;
        }

        pub fn new(&self, value: &str) -> (String, String) {
            return (self.to_str().into(), value.into());
        }
    }

    pub struct Response<'a> {
        status: &'a Status,
        body: Option<&'a str>,
        headers: &'a Headers,
    }

    impl<'a> Response<'a> {
        pub fn new(
            status: &'a Status,
            body: Option<&'a str>,
            headers: &'a Headers,
        ) -> Response<'a> {
            return Response {
                body,
                headers,
                status,
            };
        }
    }

    impl<'a> ToString for Response<'a> {
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

    #[derive(Debug, PartialEq)]
    pub enum Method {
        GET,
        HEAD,
        POST,
        PUT,
        DELETE,
        CONNECT,
        OPTIONS,
        TRACE,
        PATCH,
    }

    impl FromStr for Method {
        type Err = ();

        fn from_str(input: &str) -> Result<Method, Self::Err> {
            match input.to_uppercase().as_str() {
                "GET" => Ok(Method::GET),
                "HEAD" => Ok(Method::HEAD),
                "POST" => Ok(Method::POST),
                "PUT" => Ok(Method::PUT),
                "DELETE" => Ok(Method::DELETE),
                "CONNECT" => Ok(Method::CONNECT),
                "OPTIONS" => Ok(Method::OPTIONS),
                "TRACE" => Ok(Method::TRACE),
                "PATCH" => Ok(Method::PATCH),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug)]
    pub struct Request {
        pub method: Method,
        pub path: String,
        pub protocol_version: String,
        pub headers: Headers,
        pub body: Option<Vec<u8>>,
    }
}
