# syntax=docker/dockerfile:1.4.3

FROM busybox:1.34.1-glibc AS gs
WORKDIR /gs
ADD https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs1000/ghostscript-10.0.0-linux-x86_64.tgz gs.tar.gz
RUN tar xf gs.tar.gz

FROM debian:bullseye-20220912-slim AS latex
WORKDIR /app
RUN apt-get update && apt-get install -y perl curl
ADD https://mirrors.rit.edu/CTAN/systems/texlive/tlnet/install-tl-unx.tar.gz install-tl.tar.gz
RUN tar xf install-tl.tar.gz
COPY texlive.profile .
RUN ./install-tl-*/install-tl --profile texlive.profile --repository https://mirrors.rit.edu/CTAN/systems/texlive/tlnet
RUN ./texlive/texdir/bin/*-linux/tlmgr install \
  chemfig simplekv circuitikz xstring siunitx esint mhchem chemformula \
  tikz-cd pgfplots cancel doublestroke units physics rsfs cjhebrew \
  standalone esint-type1 pgf-blur tikzlings tikzducks \
  collection-fontsrecommended
COPY texmf.cnf texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/*-linux/pdflatex -ini -output-format pdf '&latex preamble.tex'

FROM golang:1.19.1-bullseye AS run
WORKDIR /app
COPY go.mod run.go ./
RUN go build -ldflags '-w -s' run.go

FROM pwn.red/jail:0.3.0
COPY --link --from=busybox:1.34.1-glibc / /srv
COPY --link --from=gs /gs/ghostscript-*/gs-* /srv/app/gs
COPY --link --from=latex /usr/lib/*-linux-gnu/libstdc++.so.6 /lib/*-linux-gnu/libgcc_s.so.1 /srv/lib/
COPY --link --from=latex /app/texlive /srv/app/texlive
COPY --link --from=latex /app/preamble.fmt /srv/app/preamble.fmt
COPY --link --from=run /app/run /srv/app/run
ENV JAIL_TIME=0 JAIL_PIDS=10 JAIL_MEM=100M JAIL_CPU=1000 JAIL_TMP_SIZE=20M
