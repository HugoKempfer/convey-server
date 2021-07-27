FROM rust:1.53.0 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/convey-server
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/convey-server /usr/local/bin/convey-server

ENV CONVEY-ADDR=0.0.0.0
CMD ["convey-server"]
