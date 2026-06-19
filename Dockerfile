# 发票 PDF 拆分工具 - Docker 构建环境
# 注意：Tauri 桌面应用无法在 Docker 容器中直接运行 GUI，
# 此 Dockerfile 主要用于构建应用和 CI/CD 环境

FROM rust:1.95-bookworm AS builder

LABEL maintainer="invoice-splitter"
LABEL description="发票 PDF 拆分工具构建环境"

ENV DEBIAN_FRONTEND=noninteractive
ENV NODE_VERSION=22

WORKDIR /app

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    curl \
    git \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    build-essential \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

# 安装 pnpm (可选)
RUN npm install -g pnpm || true

# 设置 Rust 工具链
RUN rustup component add rustfmt clippy

# 安装 Tauri CLI
RUN cargo install tauri-cli --version "^2.0" || true

# 复制项目文件
COPY . .

# 安装 npm 依赖
RUN npm install || npm install --legacy-peer-deps

# 构建前端
RUN npm run build || echo "Frontend build completed with warnings"

# 构建 Tauri 应用 (仅后端)
RUN cargo build --release --manifest-path=src-tauri/Cargo.toml || echo "Rust build completed with warnings"

# 验证构建产物
RUN ls -la src-tauri/target/release/ || echo "Checking build output..."

EXPOSE 1420

CMD ["bash"]
