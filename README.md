# Thema web index

Thema's web index is a queryable store of the data fetched from the web. It
stores:

- [X] pages
- [ ] request attempts (results)

These are queryable via two interfaces:

- [ ] a read only query interface with no other dependencies
- [ ] a read/append query interface which depends at present on a running
  postgres database

These interfaces are available as rust libraries (crates) or as python libraries
(bindings into the rust). They are optimised for high performance at scale; you
should feed in as many page queries as possible and let the library work out how to
optimise, at any rate as a first pass.

## Internal: Response store

!!! Warning

    Internally, the page store is actually a response store. For completeness this
    store is documented here, but this should be considered an implementation detail
    and may change at any time. Access except via the interfaces is not supported
    and may break at any point without notice.

The fundamental unit of work when using the internet is an http request, so this
is what we persist. Requests are stored one per row in parquet files in s3, with
the following naming convention:

```
s3://thema-web-index
s3://thema-web-index/cache
s3://thema-web-index/cache/2024/08
s3://thema-web-index/cache/2024/08/thema.ai.parquet
s3://thema-web-index/cache/2024/08/thema.ai.1.parquet
s3://thema-web-index/cache/2024/08/thema.ai.2.parquet
```

Formally, this is:

```
s3://{bucket_name}/cache/{year}/{month}/{domain\*}.{(part)}.parquet
```

where domain is calculated according to the logic in this repo. The part
suffixes are positive integers starting at 1 and have no meaning except to
indicate sequential writing. All parts have to be fetched every time a domain is
accessed. From time to time these files may be defragmented by rolling all part
files into one (another reason not to work directly on the store). For this to
happen the store must be locked; currently this lock is enforced via slack :D .

The `cache` subdir exists for futureproofing.

Internally, each parquet file has the schema:

* **page_url**: (String) the url of the page when we started fetching it

* **request_url**: (String) the url of this request, which may differ from the page
    url (e.g. in case of redirect)

* **status_code**: (u8) the status code of this response

* **data**: (bytes) the body of the response, in bytes

* **headers**: (String, JSON) the headers of the response, parsed to json

* **timestamp**: () The timestamp of this particular response or request

* **retry_attempt**: (u8) The attempt number for this request, starting at 0

* **is_final**: (bool) (internal) whether this is the final response in a chain.

* **request_id**: (String) (internal) a unique id for the linked list of
    responses starting with the first request and ending with the
    response whose `is_final` is true. This is an implementation
    detail used to optimise grouping. Where a query has a
    `request_id`, there is no need to filter on timestamp or url.

* **fetcher_name**: (String) the human readable name of the fetcher which
     fetched this page.

* **fetcher_version**: (String) the version of the fetcher, as reported by
  `--version`.

* **fetcher_calibre**: (u8) the [calibre](#fetcher-calibre) of this fetcher.

## Page Query

A page is requested from the web index by constructing a page query. Processes
(like discovery) which interact with the store persist page queries. A page query is
actually a union of several kinds, which imply different query
mechanisms:

!!! todo

    Proper names when the code is written.

- **Page Key** : this uniquely identifies a page in the store. It will
  always return exactly the same page. Access cost is constant. Page
  keys should be considered opaque; currently they simply consist of a url,
  timestamp and request id. They are returned by processes which access pages,
  and permit loading up the exact same page.

  Schematically a page key looks like this:

  ```python
  class PageKey:
      url: str
      timestamp: datetime
      request_id: str
  ```

- **Time aware Page Query**: this matches the nearest fetch to a point in time,
  with an acceptable window outside of which it fails to match. If a calibre is
  provided only records fetched with that calibre or higher will be returned,
  unless `calibre_strict` is true, in which case only exact matches will be used.

  Schematically a time aware page query looks like this:

  ```python
  class TimeAwarePageQuery:
      url: str
      not_before: datetime
      not_after: datetime
      timestamp: datetime
      calibre: int | None
      calibre_strict: bool = False
  ```

- **Simple Page Query**: this matches the latest page for a given url, if any is
  present. Access is expensive and grows linearly with the size of the cache. If
  a calibre is provided only records fetched with that calibre or higher will be
  returned, unless `calibre_strict` is true, an which case only exact matches
  will be used.

  Schematically a simple page query looks like this:

  ```python
  class SimplePageQuery:
      url: str
      calibre: int | None
      calibre_strict: bool = False
  ```


# Fetcher Calibre

Every fetcher has a calibre, which is a measure of how likely it is to succeed
at fetching any given target, and how expensive it will be to do so. (The
analogy is with artillery, where a higher calibre weapon has more chance of
destroying a given target, but is obviously more expensive to use.) Thus
browsers are much slower and resource intensive than httpx, but more likely to
work, and so on.

Internally calibre is an enum, but for compatibility it is an unsigned integer
between 0 (unknown) and 100 (reserved). Since in theory an increase in calibre
is an increase in the probability of loading a given page, the default behaviour
is to return pages of a given calibre *or higher*. Consumers should bear this in
mind and operate on the calibre of the returned page rather than the calibre
requested, or use `calibre_strict` (which is equally performant).
