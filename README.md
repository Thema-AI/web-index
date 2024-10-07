# Thema web index

Thema's web index is a queryable store of the data fetched from the web, and
metadata about the fetching attempt.

The formal definition of the interface and storage backend can be found in the
[latest standard](standard/latest/standard.md).

**TODO update below when the library is finished: THIS IS STALE**

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
