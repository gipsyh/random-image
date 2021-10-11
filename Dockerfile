FROM scratch
COPY target/x86_64-unknown-linux-musl/release/random-image /
COPY images/ /images
COPY Rocket.toml /
CMD ["/random-image"]