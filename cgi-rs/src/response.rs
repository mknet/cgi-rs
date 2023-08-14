use crate::{error, CGIError, Result};
use hyper::{http::HeaderValue, HeaderMap, Response};
use snafu::ResultExt;
use std::io::Write;

#[derive(Debug)]
pub struct CGIResponse<B: hyper::body::HttpBody> {
    headers: HeaderMap<HeaderValue>,
    status: String,
    reason: Option<String>,
    body: B,
}

impl<B: hyper::body::HttpBody> CGIResponse<B> {
    pub async fn write_response_to_output(self, mut output: impl Write) -> Result<()> {
        self.write_status(&mut output).await?;
        self.write_headers(&mut output).await?;
        self.write_body(&mut output).await?;

        Ok(())
    }

    async fn write_status(&self, output: &mut impl Write) -> Result<()> {
        // If a canonical reason is present, write it in the status line.
        if let Some(reason) = &self.reason {
            output
                .write(format!("Status: {} {}\n", self.status, reason).as_bytes())
                .context(error::WriteResponseSnafu)?;
        } else {
            output
                .write(format!("Status: {}\n", self.status).as_bytes())
                .context(error::WriteResponseSnafu)?;
        }
        Ok(())
    }

    async fn write_headers(&self, output: &mut impl Write) -> Result<()> {
        for (key, value) in self.headers.iter() {
            let mut header_bytes = format!("{}: ", key).into_bytes();
            header_bytes.extend(value.as_bytes());
            header_bytes.extend(b"\n");
            output
                .write(&header_bytes)
                .context(error::WriteResponseSnafu)?;
        }

        output.write(b"\n").context(error::WriteResponseSnafu)?;

        Ok(())
    }

    async fn write_body(self, output: &mut impl Write) -> Result<()> {
        let body = hyper::body::to_bytes(self.body)
            .await
            .or_else(|_| error::BuildResponseSnafu.fail())?;

        output.write(&body).context(error::WriteResponseSnafu)?;

        Ok(())
    }
}

impl<B: hyper::body::HttpBody> TryFrom<Response<B>> for CGIResponse<B> {
    type Error = CGIError;

    fn try_from(response: Response<B>) -> Result<Self> {
        let headers = response.headers().clone();
        let status = response.status().to_string();
        let reason = response.status().canonical_reason().map(|s| s.to_string());
        let body = response.into_body();
        Ok(CGIResponse {
            headers,
            status,
            reason,
            body,
        })
    }
}
