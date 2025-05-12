# 第一阶段：依赖缓存层
FROM rust:1.86-slim as cacher
WORKDIR /usr/src/app/rsendmail
COPY rsendmail/Cargo.* ./
RUN rm -f Cargo.lock && \
    mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo fetch

# 第二阶段：构建层
FROM rust:1.86-slim as builder
WORKDIR /usr/src/app/rsendmail
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY rsendmail .
RUN rm -f Cargo.lock && \
    cargo build --release --offline

# 第三阶段：运行层
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -r -s /bin/false rsendmail && \
    mkdir /data && \
    chown rsendmail:rsendmail /data

# 只复制必要的二进制文件
COPY --from=builder /usr/src/app/rsendmail/target/release/rsendmail /usr/local/bin/

USER rsendmail
WORKDIR /data
ENTRYPOINT ["rsendmail"]
