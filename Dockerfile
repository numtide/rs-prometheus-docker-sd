FROM ekidd/rust-musl-builder as builder
WORKDIR /home/rust/
COPY . .
RUN cargo test
RUN cargo build --release

FROM scratch

ARG BUILD_DATE="1970-01-01T00:00:00Z"
ARG VCS_REF="local"
ARG VERSION="SNAPSHOT"

# Build-time metadata as defined at http://label-schema.org
LABEL org.label-schema.build-date=${BUILD_DATE} \
      org.label-schema.name="rs-prometheus-docker-sd" \
      org.label-schema.description="Prometheus Service Discovery for Docker Container." \
      org.label-schema.vendor="NumTide Ltd <info@numtide.com>" \
      org.label-schema.url="https://hub.docker.com/r/numtide/rs-prometheus-docker-sd" \
      org.label-schema.vcs-ref=${VCS_REF} \
      org.label-schema.vcs-url="https://github.com/numtide/rs-prometheus-docker-sd" \
      org.label-schema.usage="https://github.com/numtide/rs-prometheus-docker-sd/blob/master/README.md" \
      org.label-schema.version=${BUILD_DATE} \
      org.label-schema.schema-version="1.0" \
      org.label-schema.docker.cmd="docker run -d -v /var/run/docker.sock:/var/run/docker.sock numtide/rs-prometheus-docker-sd:latest"

# Build-time metadata as defined at https://github.com/opencontainers/image-spec/blob/master/annotations.md
LABEL org.opencontainers.image.ref.name="numtide/rs-prometheus-docker-sd" \
      org.opencontainers.image.created=$BUILD_RFC3339 \
      org.opencontainers.image.authors="NumTide <info@numtide.com>" \
      org.opencontainers.image.documentation="https://github.com/numtide/rs-prometheus-docker-sd/blob/master/README.md" \
      org.opencontainers.image.description="Prometheus Service Discovery for Docker Container." \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.source="https://github.com/numtide/rs-prometheus-docker-sd" \
      org.opencontainers.image.revision=$VCS_REF \
      org.opencontainers.image.version=$VERSION \
      org.opencontainers.image.url="https://hub.docker.com/r/numtide/rs-prometheus-docker-sd"

COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/rs-promotheus-docker-sd /app
VOLUME /rs-prometheus-docker-sd
ENTRYPOINT ["/app"]
