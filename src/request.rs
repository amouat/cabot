//! HTTP Request handling.
//!
//! # Example
//! ```
//! use cabot::RequestBuilder;
//!
//! let request = RequestBuilder::new("http://localhost/")
//!     .set_http_method("POST")
//!     .add_header("Content-Type: application/json")
//!     .set_body("{}")
//!     .build()
//!     .unwrap();
//!     let attempt = "POST / HTTP/1.1\r\nContent-Type: \
//!                    application/json\r\nHost: localhost\r\nConnection: \
//!                    close\r\nContent-Length: 2\r\n\r\n{}";
//! assert_eq!(request.to_string(), attempt.to_string());
//! ```

use url::{self, Url};

use results::{CabotResult, CabotError};

/// An HTTP Request representation.
///
/// Request is build using [RequestBuilder](../request/struct.RequestBuilder.html)
/// and them consume by the [Client](../client/struct.Client.html)
/// to perform the query.
pub struct Request {
    host: String,
    port: u16,
    authority: String,
    is_domain: bool,
    scheme: String,
    http_method: String,
    request_uri: String,
    http_version: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl Request {
    fn new(host: String,
           port: u16,
           authority: String,
           is_domain: bool,
           scheme: String,
           http_method: String,
           request_uri: String,
           http_version: String,
           headers: Vec<String>,
           body: Option<String>)
           -> Request {
        Request {
            host: host,
            port: port,
            authority: authority,
            is_domain: is_domain,
            scheme: scheme,
            http_method: http_method,
            request_uri: request_uri,
            http_version: http_version,
            headers: headers,
            body: body,
        }
    }

    /// The HTTP verb used to perform the request,
    /// such as GET, POST, ...
    pub fn http_method(&self) -> &str {
        self.http_method.as_str()
    }

    /// The HTTP Body of the request.
    pub fn body(&self) -> Option<&str> {
        match self.body {
            Some(ref payload) => Some(payload.as_str()),
            None => None,
        }
    }

    /// The Version of the HTTP to perform the request.
    pub fn http_version(&self) -> &str {
        self.http_version.as_str()
    }

    /// The server name to connect. can be a name to resolve or an IP address.
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// The TCP server port to connect.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The authority part of the url (`host`:`port`).
    pub fn authority(&self) -> &str {
        self.authority.as_str()
    }

    /// The protocol scheme, can be http or https.
    pub fn scheme(&self) -> &str {
        self.scheme.as_str()
    }

    /// The URI to send, something like a PATH_INFO and a querystring.
    pub fn request_uri(&self) -> &str {
        self.request_uri.as_str()
    }

    /// The String representation of the query to send to the server.
    pub fn to_string(&self) -> String {
        let mut resp = format!("{} {} {}\r\n",
                               self.http_method(),
                               self.request_uri(),
                               self.http_version());
        if self.headers.len() > 0 {
            resp.push_str(self.headers.as_slice().join("\r\n").as_str());
            resp.push_str("\r\n");
        }
        if self.is_domain {
            resp.push_str(format!("Host: {}\r\n", self.host()).as_str());
        }
        resp.push_str("Connection: close\r\n");
        if let Some(payload) = self.body() {
            resp.push_str(format!("Content-Length: {}\r\n", payload.len()).as_str());
            resp.push_str("\r\n");
            resp.push_str(payload);
        } else {
            resp.push_str("\r\n");
        }
        resp
    }
}

/// Construct [Request](../request/struct.Request.html)
pub struct RequestBuilder {
    http_method: String,
    url: Result<Url, url::ParseError>,
    http_version: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl RequestBuilder {
    /// Create a new RequestBuilder with the given url.
    ///
    /// `url` will be parsed to get the host to contact and the uri to send.
    /// When building a request object after creating the builder,
    /// the http method will be `GET` with neither header nor body and
    /// the http version will be `HTTP/1.1`
    pub fn new(url: &str) -> Self {
        let url = url.parse::<Url>();
        RequestBuilder {
            http_method: "GET".to_owned(),
            url: url,
            http_version: "HTTP/1.1".to_owned(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Replace the url in case the RequestBuilder is used many time
    /// for multiple query.
    pub fn set_url(mut self, url: &str) -> Self {
        self.url = url.parse::<Url>();
        self
    }

    /// Set the http method such as `GET` `POST`. Default value is `GET`.
    pub fn set_http_method(mut self, http_method: &str) -> Self {
        self.http_method = http_method.to_owned();
        self
    }

    /// Set the protocol version to use.. Default value is `HTTP/1.1`.
    pub fn set_http_version(mut self, http_version: &str) -> Self {
        self.http_version = http_version.to_owned();
        self
    }

    /// Add a HTTP header.
    pub fn add_header(mut self, header: &str) -> Self {
        self.headers.push(header.to_owned());
        self
    }

    /// Add many headers.
    pub fn add_headers(mut self, headers: &[&str]) -> Self {
        for header in headers {
            self.headers.push(header.to_string());
        }
        self
    }

    /// Set a body to send in the query. By default a query has no body.
    pub fn set_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_owned());
        self
    }

    /// Construct the [Request](../request/struct.Request.html).
    /// To perform the query, a [Client](../client/struct.Client.html)
    /// has to be created.
    ///
    /// Errors:
    ///
    ///   - CabotError::ParseUrlError in case the `url` is not parsable
    ///   - CabotError::OpaqueUrlError in case the `url` is parsed but miss informations such as hostname.
    ///
    pub fn build(&self) -> CabotResult<Request> {
        if let Err(ref err) = self.url {
            return Err(CabotError::UrlParseError(err.clone()));
        }
        let url = self.url.as_ref().unwrap().clone();

        let host = url.host_str();
        if host.is_none() {
            return Err(CabotError::OpaqueUrlError("Unable to find host".to_string()));
        }
        let host = host.unwrap();

        let port = url.port_or_known_default();
        if port.is_none() {
            return Err(CabotError::OpaqueUrlError("Unable to determine a port".to_string()));
        }
        let port = port.unwrap();

        let query = url.query();
        let mut request_uri = url.path().to_owned();
        if let Some(querystring) = query {
            request_uri.push_str("?");
            request_uri.push_str(querystring);
        }
        let mut is_domain = true;
        if url.domain().is_none() {
            is_domain = false;
        }

        Ok(Request::new(host.to_owned(),
                        port,
                        format!("{}:{}", host, port),
                        is_domain,
                        url.scheme().to_owned(),
                        self.http_method.clone(),
                        request_uri,
                        self.http_version.clone(),
                        self.headers.clone(),
                        match self.body {
                            Some(ref body) => Some(body.clone()),
                            None => None,
                        }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_request_to_string() {
        let request = Request::new("127.0.0.1".to_owned(),
                                   80,
                                   "127.0.0.1:80".to_owned(),
                                   false,
                                   "http".to_owned(),
                                   "GET".to_owned(),
                                   "/path?query".to_owned(),
                                   "HTTP/1.1".to_owned(),
                                   Vec::new(),
                                   None);
        let attempt = "GET /path?query HTTP/1.1\r\nConnection: close\r\n\r\n";
        assert_eq!(request.to_string(), attempt);
    }

    #[test]
    fn test_get_request_wiht_host_to_string() {
        let request = Request::new("localhost".to_owned(),
                                   80,
                                   "localhost:80".to_owned(),
                                   true,
                                   "http".to_owned(),
                                   "GET".to_owned(),
                                   "/path?query".to_owned(),
                                   "HTTP/1.1".to_owned(),
                                   Vec::new(),
                                   None);
        let attempt = "GET /path?query HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        assert_eq!(request.to_string(), attempt);
    }

    #[test]
    fn test_get_request_with_headers_to_string() {
        let request = Request::new("localhost".to_owned(),
                                   80,
                                   "localhost:80".to_owned(),
                                   true,
                                   "http".to_owned(),
                                   "GET".to_owned(),
                                   "/path?query".to_owned(),
                                   "HTTP/1.1".to_owned(),
                                   vec!["Accept-Language: fr".to_owned(),
                                        "Accept-Encoding: gzip".to_owned()],
                                   None);
        let attempt = "GET /path?query HTTP/1.1\r\nAccept-Language: fr\r\nAccept-Encoding: \
                       gzip\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        assert_eq!(request.to_string(), attempt);
    }

    #[test]
    fn test_post_request_with_headers_to_string() {
        let request = Request::new("localhost".to_owned(),
                                   80,
                                   "localhost:80".to_owned(),
                                   true,
                                   "http".to_owned(),
                                   "POST".to_owned(),
                                   "/".to_owned(),
                                   "HTTP/1.1".to_owned(),
                                   vec!["Accept-Language: fr".to_owned(),
                                        "Content-Type: application/json".to_owned()],
                                   Some("{}".to_owned()));
        let attempt = "POST / HTTP/1.1\r\nAccept-Language: fr\r\nContent-Type: \
                       application/json\r\nHost: localhost\r\nConnection: \
                       close\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(request.to_string(), attempt);
    }

    #[test]
    fn test_request_builder_simple() {
        let request = RequestBuilder::new("http://localhost/")
            .build()
            .unwrap();
        assert_eq!(request.host(), "localhost".to_string());
        assert_eq!(request.scheme(), "http".to_string());
        assert_eq!(request.body, None);
        assert_eq!(request.http_method(), "GET".to_string());
        assert_eq!(request.http_version(), "HTTP/1.1".to_string());
        let headers: Vec<String> = Vec::new();
        assert_eq!(request.headers, headers);
    }

    #[test]
    fn test_request_builder_complete() {
        let builder = RequestBuilder::new("http://localhost/")
            .set_http_method("POST")
            .set_http_version("HTTP/1.0")
            .add_header("Content-Type: application/json")
            .add_headers(&["Accept-Encoding: deflate", "Accept-Language: fr"])
            .set_body("{}");
        let request = builder.build().unwrap();
        assert_eq!(request.host(), "localhost".to_string());
        assert_eq!(request.body, Some("{}".to_string()));
        assert_eq!(request.scheme(), "http".to_string());
        assert_eq!(request.http_method(), "POST".to_string());
        assert_eq!(request.request_uri(), "/");
        assert_eq!(request.http_version(), "HTTP/1.0".to_string());
        assert_eq!(request.headers,
                   vec!["Content-Type: application/json".to_string(),
                        "Accept-Encoding: deflate".to_string(),
                        "Accept-Language: fr".to_string()]);

        let builder = builder.set_url("http://[::1]/path");
        let request = builder.build().unwrap();
        assert_eq!(request.host(), "[::1]".to_string());
        assert_eq!(request.request_uri(), "/path");
        assert_eq!(request.body, Some("{}".to_string()));
        assert_eq!(request.scheme(), "http".to_string());
        assert_eq!(request.http_method(), "POST".to_string());
        assert_eq!(request.http_version(), "HTTP/1.0".to_string());
        assert_eq!(request.headers,
                   vec!["Content-Type: application/json".to_string(),
                        "Accept-Encoding: deflate".to_string(),
                        "Accept-Language: fr".to_string()]);

        let builder = builder.set_url("not_an_url");
        let err = builder.build();
        assert!(err.is_err());
    }

}
