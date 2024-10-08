use tldextract_rs::{ExtractResult, Source, SuffixList, TLDExtract};
use url::Url;
pub struct Domain {
    inner: String,
}
use anyhow::{Context, Result};

type DomainResult = Result<Domain>;

pub struct Extractor {
    extractor: TLDExtract,
}

impl Extractor {
    pub(crate) fn new() -> Self {
        let suffix_list = SuffixList::new(Source::Snapshot, false, None);
        let extractor =
            TLDExtract::new(suffix_list, true).expect("Snapshot vendored with extractor");
        Self { extractor }
    }

    pub(crate) fn domain(&mut self, url: &Url) -> DomainResult {
        let result = self
            .extractor
            .extract(url.host_str().context("Host str")?)?;
        Domain::new(result)
    }
}

impl Domain {
    fn new(result: ExtractResult) -> DomainResult {
        Ok(Self {
            inner: result.registered_domain.context("registered domain")?,
        })
    }

    fn into_string(self) -> String {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::tld("http://thema.ai", "thema.ai")]
    #[case::long_domain("http://foo.bar.thema.ai", "thema.ai")]
    #[case::path("http://thema.ai/foo/bar.html", "thema.ai")]
    #[case::query_params("http://thema.ai/?foo=bar&bar=baz", "thema.ai")]
    #[case::all("http://foo.bar.thema.ai/foobar?foo=bar&bar=baz", "thema.ai")]
    #[case::long_tld("http://local.nhs.uk", "local.nhs.uk")]
    #[case::long("http://mirrors.tuna.tsinghua.edu.cn", "tsinghua.edu.cn")]
    fn domain_computed_correctly(#[case] url: &str, #[case] domain: &str) -> Result<()> {
        let mut extractor = Extractor::new();
        assert_eq!(
            extractor.domain(&url.parse()?)?.into_string(),
            domain.to_string()
        );
        Ok(())
    }

    #[rstest]
    #[case::ip("http://192.168.1.1")]
    fn impossible_domain_rejected(#[case] url: &str) -> Result<()> {
        let mut extractor = Extractor::new();
        assert!(extractor.domain(&url.parse()?).is_err());

        Ok(())
    }
}
