# syntax=docker/dockerfile:1.5.2

FROM debian:bullseye-20230411-slim AS mupdf
WORKDIR /app
RUN apt-get update && apt-get install -y build-essential
ADD https://mupdf.com/downloads/archive/mupdf-1.22.0-source.tar.gz mupdf.tar.gz
RUN mkdir mupdf && tar xf mupdf.tar.gz --strip-components 1 -C mupdf && \
  cd mupdf && make HAVE_X11=no HAVE_GLUT=no -j$(nproc)

FROM debian:bullseye-20230411-slim AS latex
WORKDIR /app
RUN apt-get update && apt-get install -y perl curl
ADD https://mirrors.rit.edu/CTAN/systems/texlive/tlnet/install-tl-unx.tar.gz install-tl.tar.gz
RUN tar xf install-tl.tar.gz
COPY texlive.profile .
RUN ./install-tl-*/install-tl --profile texlive.profile --repository https://mirrors.rit.edu/CTAN/systems/texlive/tlnet
RUN ./texlive/texdir/bin/*-linux/tlmgr install \
  cancel chemfig chemformula circuitikz cjhebrew collection-fontsrecommended \
  doublestroke esint esint-type1 mhchem pgf-blur pgfplots physics rsfs \
  simplekv siunitx standalone tikz-cd tikzducks tikzlings units xstring
COPY texmf.cnf texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/*-linux/pdflatex -ini -output-format pdf '&latex preamble.tex'

FROM rust:1.69.0-slim-bullseye AS run
WORKDIR /app
COPY Cargo.lock Cargo.toml main.rs ./
RUN cargo build --release

FROM pwn.red/jail:0.3.1
COPY --link --from=busybox:1.36.0-glibc / /srv
COPY --link --from=mupdf /app/mupdf/build/release/mutool /srv/app/
COPY --link --from=latex /usr/lib/*-linux-gnu/libstdc++.so.6 /lib/*-linux-gnu/libgcc_s.so.1 /lib/*-linux-gnu/libdl.so.2 /srv/lib/
COPY --link --from=latex /app/texlive /srv/app/texlive
COPY --link --from=latex /app/preamble.fmt /srv/app/
COPY --link --from=run /app/target/release/run /srv/app/
ENV JAIL_TIME=0 JAIL_PIDS=10 JAIL_MEM=100M JAIL_CPU=1000 JAIL_TMP_SIZE=20M
