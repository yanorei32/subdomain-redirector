FROM rust:1.98.1 as build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
RUN cargo new subdomain-redirector
COPY LICENSE Cargo.toml Cargo.lock /usr/src/subdomain-redirector/
WORKDIR /usr/src/subdomain-redirector
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN	cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

RUN cargo build --release
COPY src/ /usr/src/subdomain-redirector/src/

RUN touch src/* && cargo build --release

FROM debian:bookworm-slim

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/subdomain-redirector/CREDITS \
	/usr/src/subdomain-redirector/LICENSE \
	/usr/share/licenses/subdomain-redirector/

COPY --chown=root:root --from=build-env \
	/usr/src/subdomain-redirector/target/release/subdomain-redirector \
	/usr/bin/subdomain-redirector

CMD ["/usr/bin/subdomain-redirector"]
