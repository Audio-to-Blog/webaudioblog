FROM rust:1.68 as build-env
WORKDIR /app
COPY ./api /app
RUN ls 
RUN cargo build --release

RUN ls
RUN cd /app/target/release && ls

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/api /
CMD ["./api"]