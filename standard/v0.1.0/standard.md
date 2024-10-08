# Web Index Standard
**Version: v0.1.0**

The web index stores:

- fetched pages, i.e. GET requests
- HEAD requests (used in matching)
and:
- metadata about fetches, including failed attempts

Metadata is a strict superset of other data: there exists a metadata record for
every data record, and also for failed attempt to generate data records,
i.e. failed fetches.

The web index is provided as an API (currently written in rust, with first-class
python bindings). This API enables querying (retrieving data and metadata) and
inserting records. There is no supported method of deletion: the web index is an
append-only store.

Although this API is the only supported way to interact with the store, most of
the design effort has been focused on the actual storage backend. This standard
thus defines both, with the caveat that direct usage of the stored data is
unsupported. Further revisions to this specification may update the backend to
improve performance or add functionality, but should not remove any
functionality from the api. This guarantee will become a strict promise once
this specification passes v1.0.0.

## Fetcher Calibre

Every fetcher has a calibre, which is a measure of how likely it is to succeed
at fetching any given target, and how expensive it will be to do so. (The
analogy is with artillery, where a higher calibre weapon has more chance of
destroying a given target, but is obviously more expensive to use.) Thus
browsers are much slower and resource intensive than httpx, but more likely to
work, and so on.

Internally calibre is an enum, but for compatibility it is an unsigned integer
between 0 (unknown) and 100 (reserved). Since in theory an increase in calibre
is an increase in the probability of fetching a given url, the default behaviour
when retrieving data with a non-deterministic query is to return data fetched
with a given calibre *or higher*. Consumers should bear this in mind and operate
on the calibre of the returned data rather than the calibre requested, or use
`calibre_strict` (which is equally performant).

## API

The web index is a queryable append-only store. As such all access to it is
mediated by queries. Interfaces are currently provided in python and rust, but
queries are intended to be serialised (proabably to parquet files) and passed
between stages. To facilitate this, queries are flat data structures with
defined serialised types. This is the form shown below; the form used in the
libraries has the same names for members, but uses natural types (for example, a
timestamp is a `datetime.datetime` object in python). Both libraries support
deserialising and serialising their native objects into the form described here
(for example, in order to save a list of queries to the registry).

### Insertion

The simplest query is an insertion request, which looks like this:

- **type**: (String) the type of record being inserted, one of "head" or "get"
- **url**: (String) the url to which the request was made
- **timestamp**: (String; ISO8601 datetime) the timestamp at which the attempt *began*

This must be accompanied by a payload of a `metadata` record and an array of
`data` records as defined below, with the exception that the `request_id` column
should not be supplied. If the attempt failed the data array should be empty.

The api ingests the data and returns a deterministic retrieval query.

### Retrieval

#### Deterministic query

A deterministic query will always return exactly the same data. Since the web
index is append-only, this property holds indefinitely.

The query looks like this:

- **type**: (String) the type of record (data or metadata), one of "head",
  "head-metadata", "get" or "get-metadata"
- **url**: (String) the url to which the request was made
- **timestamp**: (String; ISO8601 datetime) the timestamp at which the attempt
  *began*
- **request_id** (String; internal) the opaque id of this request, obtained from
  a previous query (for example, returned after inserting a record). This value
  is strictly opaque and no inferences should be drawn from its construction.

#### Simple query

This matches the latest page for a given url, if any is present. Access is
expensive and grows linearly with the size of the cache. If a calibre is
provided only records fetched with that calibre or higher will be returned,
unless `calibre_strict` is true, an which case only exact matches will be used.

A simple page query looks like this:

- **type**: (String) the type of record (data or metadata), one of "head",
  "head-metadata", "get" or "get-metadata"
- **url**: (String) the url to which the request was made
- **calibre**: (u8) the [calibre](#fetcher-calibre) of the fetcher used
- **calibre_strict**: (bool) whether to match records fetched with a superior calibre

#### Time bounded query

This matches the nearest fetch to a point in time, with an acceptable window
outside of which it fails to match. If a calibre is provided only records
fetched with that calibre or higher will be returned, unless `calibre_strict` is
true, in which case only exact matches will be used.

A time bounded query looks like this:

- **type**: (String) the type of record (data or metadata), one of "head",
  "head-metadata", "get" or "get-metadata"
- **url**: (String) the url to which the request was made
- **not_before**: (String; ISO8601 datetime) attempts made strictly before this
  timestamp will not be matched
- **not_after**: (String; ISO8601 datetime) attempts made strictly after this
  timestamp will not be matched
- **calibre**: (u8) the [calibre](#fetcher-calibre) of the fetcher used
- **calibre_strict**: (bool) whether to match records fetched with a superior
  calibre. In the libraries this defaults to true.

### Presence

The same queries used for retrieval can be passed to the presence functionality,
which returns a boolean indicating whether any records match the query. This can
be assumed to be faster than retrieving the whole record and then casting to bool.

### List/Search

In the future a search like interface may be added to allow discovering what
data is in fact in the web index.

## Backend

All data is stored in parquet files in s3, with the following naming convention:

```
thema-web-index/
├── head
│   └── 2024
│       └── 08
│           ├── thema.ai.1234-1234-1234-1234.parquet
│           └── thema.ai.parquet
├── head-metadata
│   └── 2024
│       └── 08
│           ├── thema.ai.1234-1234-1234-1234.parquet
│           └── thema.ai.parquet
├── get
│   └── 2024
│       └── 08
│           ├── thema.ai.1234-1234-1234-1234.parquet
│           ├── thema.ai.2345-2345-2345-2345.parquet
│           └── thema.ai.parquet
└── get-metadata
    └── 2024
        └── 08
            ├── thema.ai.1234-1234-1234-1234.parquet
            └── thema.ai.parquet
```

The path consists of:
- a top-level dir, one of:
  + **head**: HEAD requests
  + **head-metadata**: metadata about attempts to make head requests
  + **get**: pages, i.e. GET requests
  + **get-metadata**: metadta about attempts to make get requests
- the year in which the request was started
- the month in which the request was started
- one or more files in the form `<DOMAIN>.<PART>.parquet`

where domain is calculated as below. The part suffixes are uuids and have no
meaning except to prevent collisions when writing. All parts have to be fetched
every time a domain is accessed. From time to time these files may be
defragmented by rolling all part files into one (another reason not to work
directly on the store), creating the files shown above without a part
component. For this to happen the store must be locked; currently this lock is
enforced via slack :D .

### Schema

We persist response chains. The GET data has the following schema:

- **url**: (String) the url we set out to fetch
- **request_url**: (String) the url of this request, which may differ from the initial
    url (e.g. in case of redirect)
- **status_code**: (u8) the status code of this response
- **data**: (bytes) the body of the response, as raw binary
- **headers**: (String, JSON) the headers of the response, parsed to json
- **timestamp**: (String, ISO8601 datetime) The timestamp of this particular
  response or request
- **retry_attempt**: (u8) The attempt number for this request, starting at 0
- **is_final**: (bool) (internal) whether this is the final response in a chain.
- **request_id**: (String) (internal) a unique id for the linked list of
    responses starting with the first request and ending with the
    response whose `is_final` is true. This is an implementation
    detail used to optimise grouping. Where a query has a
    `request_id`, there is no need to filter on timestamp or url.
- **fetcher_name**: (String) the human readable name of the fetcher which
     fetched this url.
- **fetcher_version**: (String) the version of the fetcher, as reported by
  `--version`.
- **fetcher_calibre**: (u8) the [calibre](#fetcher-calibre) of this fetcher.

The HEAD data is exactly the same, except that there is no "body" column.

The metadata files all have the following schema:

- **state:** (String) the state of this request, defined below
- **url**: (String) the url we set out to fetch
- **logs**: (String | Null) logs from the fetcher for this attempt
- **traceback**: (String | Null) Error traceback if this attempt failed
- **run_time**: (float | Null) the time taken for this attempt, in S

### Attempt States
An attempt can finish in a number of states:

| state           | meaning                                                                | retryable? | retryable with escalation? |
|-----------------|------------------------------------------------------------------------|------------|----------------------------|
| success         | attempt succeeded                                                      | n/a        | n/a                        |
| timeout         | attempt timed out                                                      | yes        | yes                        |
| unreachable     | unable to reach the resource                                           | yes        | yes                        |
| ssl-error       | ssl connection could not be established (e.g. due to expired cert)     | yes        | yes                        |
| low-quality     | retrieved data judged to be low quality e.g. a "please enable JS" page | no         | yes                        |
| blocked         | we were blocked when attempting to retrieve the page                   | no         | yes                        |
| unauthorised    | the resource required us to be logged in                               | no         | no                         |
| retryable-error | catch-all "please retry"; fetchers should prefer a specific state      | yes        | yes                        |
| escalate        | catch-all "please escalate"; fetchers should prefer a specific state   | no         | yes                        |
| error           | attempt failed                                                         | no         | no                         |
