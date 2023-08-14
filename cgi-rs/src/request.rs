use crate::{error, CGIError, MetaVariable, MetaVariableKind, Result};
use hyper::{Body as HttpBody, Request};
use snafu::ResultExt;
use std::io::{stdin, Read};

pub struct CGIRequest {
    pub request_body: HttpBody,
}

impl CGIRequest {
    pub fn from_env() -> Result<Self> {
        let content_length = MetaVariableKind::ContentLength
            .from_env()
            .map(|content_length| {
                content_length
                    .as_str()
                    .and_then(|s| s.parse().context(error::InvalidContentLengthSnafu))
            })
            .transpose()?
            .unwrap_or_default();

        let request_body = HttpBody::from(Self::request_body_from_env(content_length)?);
        Ok(Self { request_body })
    }

    pub fn var(&self, kind: MetaVariableKind) -> Option<MetaVariable> {
        kind.from_env()
    }

    fn try_var(&self, kind: MetaVariableKind) -> Result<MetaVariable> {
        kind.try_from_env()
    }

    fn request_body_from_env(content_length: usize) -> Result<Vec<u8>> {
        let mut request_body = vec![0u8; content_length];
        stdin()
            .read_exact(&mut request_body)
            .context(error::ReadRequestBodySnafu)
            .and(Ok(request_body))
    }

    pub fn uri(&self) -> Result<String> {
        // Some CGI implementations (e.g. Apache) set REQUEST_URI, which isn't in the RFC
        self.var(MetaVariableKind::RequestUri)
            .map(|uri| Ok(uri.as_str()?.to_string()))
            .unwrap_or_else(|| {
                let script_name = MetaVariableKind::ScriptName.try_from_env()?;
                let query_string = MetaVariableKind::QueryString.try_from_env()?;
                Ok(format!(
                    "{}?{}",
                    script_name.as_str()?,
                    query_string.as_str()?
                ))
            })
    }
}

macro_rules! try_set_headers {
    ($request_builder:expr, $cgi_request:expr, $([$header:expr, $value:expr]),* $(,)?) => {
        $(
            if let Some(value) = $cgi_request.var($value) {
                $request_builder = $request_builder.header($header, value.as_bytes());
            }
        )*
    };
}

impl TryFrom<CGIRequest> for Request<HttpBody> {
    type Error = CGIError;

    fn try_from(cgi_request: CGIRequest) -> Result<Self> {
        let mut request_builder = Request::builder()
            .method(
                cgi_request
                    .try_var(MetaVariableKind::RequestMethod)?
                    .as_bytes(),
            )
            .uri(cgi_request.uri()?);

        try_set_headers!(
            request_builder,
            cgi_request,
            ["Content-Length", MetaVariableKind::ContentLength],
            ["Accept", MetaVariableKind::HttpAccept],
            ["Host", MetaVariableKind::HttpHost],
            ["User-Agent", MetaVariableKind::HttpUserAgent],
        );

        request_builder
            .body(cgi_request.request_body)
            .context(error::RequestParseSnafu)
    }
}
