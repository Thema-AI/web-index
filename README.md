# Thema web index

Thema's web index is a queryable store of the data fetched from the web, and
metadata about the fetching attempt.

The formal definition of the interface and storage backend can be found in the
[latest standard](standard/latest/standard.md).


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

# Return types

The query interfaces return `Vec<Option<Page>>` in rust and `list[Page | None]`
in python.

# Examples

!!! TODO

    update when we have actual code

To fetch the latest page for any of a thousand urls, synchronously:

```python
from web_index.read_only import query
from web_index.models import SimplePageQuery

urls = ["http://example.com"] * 1_000 # replace with real data
queries = [SimplePageQuery(url=url) for url in urls]
results = query(queries)
print(results)
```

The same, asynchronously (note that the underlying rust code always runs in an
asynchronous loop: the api here is just to free up the python event loop to do
something else):
```python
from web_index.read_only import async_query
from web_index.models import SimplePageQuery

urls = ["http://example.com"] * 1_000 # replace with real data
queries = [SimplePageQuery(url=url) for url in urls]
results = await async_query(queries)
print(results)
```
