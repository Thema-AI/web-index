#syntax=docker/dockerfile:1.4
ARG alpine_version=3.20.3

FROM alpine:${alpine_version}
COPY --link /dist/web-index /app/web-index
ENTRYPOINT ["/app/web-index"]


