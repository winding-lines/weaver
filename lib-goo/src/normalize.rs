use lib_error::*;
use std::borrow::Cow;
use url::Url;

// Return a canonical URL for the given input.
pub fn normalize_url<'a>(input: &'a str) -> Result<Cow<'a, str>> {
    // fast check
    if !input.contains('?') && !input.contains('#') {
        // Return the input as Borrowed.
        return Ok(input.into());
    }
    let mut url = Url::parse(input).map_err(|_| "cannot parse url")?;
    if url.fragment().is_some() {
        url.set_fragment(None);
    }
    if url.query().is_some() {
        url.set_query(None);
    }
    {
        let mut segments = url.path_segments_mut().map_err(|_| "normalize url paths")?;
        segments.pop_if_empty();
    }

    // Build a string from the URL object then wrap it into an Owned Cow.
    let as_str = url.into_string();
    Ok(as_str.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize_url("https://foo/bar").unwrap(), "https://foo/bar");
        assert_eq!(
            normalize_url("https://foo/bar#head?a").unwrap(),
            "https://foo/bar"
        );
        assert_eq!(
            normalize_url("https://foo/bar?a").unwrap(),
            "https://foo/bar"
        );
    }
}
