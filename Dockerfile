FROM debian:bullseye-20220822-slim AS gs
WORKDIR /app
RUN apt-get update && \
  apt-get install -y curl && \
  curl -fL https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs9550/ghostscript-9.55.0-linux-x86_64.tgz | tar xzoC . --strip-components 1

FROM debian:bullseye-20220822-slim AS latex
WORKDIR /app
COPY texlive.profile .
RUN apt-get update && \
  apt-get install -y curl perl && \
  mkdir install-tl && \
  curl -f https://mirrors.rit.edu/CTAN/systems/texlive/tlnet/install-tl-unx.tar.gz | tar xzoC install-tl --strip-components 1 && \
  ./install-tl/install-tl --profile texlive.profile --repository https://mirrors.rit.edu/CTAN/systems/texlive/tlnet
RUN ./texlive/texdir/bin/*-linux/tlmgr install \
  chemfig simplekv circuitikz xstring siunitx esint mhchem chemformula \
  tikz-cd pgfplots cancel doublestroke units physics rsfs cjhebrew \
  standalone esint-type1 pgf-blur tikzlings tikzducks \
  collection-fontsrecommended booktabs
COPY texmf.cnf texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/*-linux/pdflatex -ini -output-format pdf '&latex preamble.tex'

FROM golang:1.19.0-bullseye AS run
WORKDIR /app
COPY go.mod run.go ./
RUN go build -ldflags '-w -s' run.go

FROM pwn.red/jail:0.3.0
COPY --from=busybox:1.34.1-glibc / /srv
COPY --from=gs /usr/lib/*-linux-gnu/libstdc++.so.6 /lib/*-linux-gnu/libgcc_s.so.1 /srv/lib/
COPY --from=gs /app/gs-* /srv/app/gs
COPY --from=latex /app/texlive /srv/app/texlive
COPY --from=latex /app/preamble.fmt /srv/app
COPY --from=run /app/run /srv/app
ENV JAIL_TIME=0 JAIL_PIDS=10 JAIL_MEM=50M JAIL_CPU=800 JAIL_TMP_SIZE=20M
