FROM rust:1.68 as build-env
WORKDIR /app
COPY ./api /app
RUN ls 
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/api /
CMD ["./api"]