FROM rust:1.87.0 
# FROM rust
RUN USER=root apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install git curl g++ build-essential libssl-dev pkg-config && \
    apt-get -y install software-properties-common && \
    apt-get update 

RUN  apt-get -y  install openssh-server
RUN  apt-get -y  install openssh-client    

COPY ./ ./relayer-order-api

WORKDIR /relayer-order-api
RUN rm Cargo.lock
RUN cargo build --release

EXPOSE 3032
ENTRYPOINT ["/relayer-order-api/target/release/relayer-order-api"]
