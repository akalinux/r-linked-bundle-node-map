FROM mcr.microsoft.com/devcontainers/typescript-node

RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
   apt-get update && apt-get install -y \
    curl \
    build-essential \
    vim wget llvm ispell cargo 
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
   apt-get update && apt-get install -y \
   rustup

RUN /usr/local/bin/corepack enable
RUN COREPACK_ENABLE_DOWNLOAD_PROMPT=0 yarn set version stable
RUN cd /;curl -L https://git.io/misspell | bash
USER node
RUN COREPACK_ENABLE_DOWNLOAD_PROMPT=0 yarn set version stable
RUN rustup default stable 
RUN rustup update
RUN cargo install --locked cargo-vet
USER root
RUN curl https://wasm-bindgen.github.io/wasm-pack/installer/init.sh -sSf | sh
#RUN rustup docs --book 
 
ENV PATH=/usr/local/go/bin:$PATH
CMD ["/bin/bash"]
