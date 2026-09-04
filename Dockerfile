# syntax=docker/dockerfile:1.2
FROM ghcr.io/radiosilence/nano-web:latest

LABEL org.opencontainers.image.source="https://github.com/radiosilence/queenshead" \
  org.opencontainers.image.url="https://queensheadstratford.london" \
  org.opencontainers.image.title="queenshead" \
  org.opencontainers.image.description="The Queen's Head, Stratford E15" \
  org.opencontainers.image.vendor="James Cleveland" \
  org.opencontainers.image.licenses=""

COPY dist /public
ENV PORT=3000
EXPOSE 3000
