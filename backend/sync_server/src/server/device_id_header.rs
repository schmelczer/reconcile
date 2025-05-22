use axum_extra::headers;
use headers::{Header, HeaderName, HeaderValue};

pub struct DeviceIdHeader(pub String);

pub static DEVICE_ID_HEADER_NAME: HeaderName = HeaderName::from_static("device-id");

impl Header for DeviceIdHeader {
    fn name() -> &'static HeaderName { &DEVICE_ID_HEADER_NAME }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        Ok(DeviceIdHeader(
            value
                .to_str()
                .map_err(|_| headers::Error::invalid())?
                .to_owned(),
        ))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(Box::leak(self.0.to_string().into_boxed_str()));

        values.extend(std::iter::once(value));
    }
}
