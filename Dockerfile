FROM debian:11.0-slim AS latex
WORKDIR /app
COPY texlive.profile .
RUN apt-get update && \
    apt-get install -y curl perl && \
    mkdir install-tl && \
    curl -fsS https://mirrors.rit.edu/CTAN/systems/texlive/tlnet/install-tl-unx.tar.gz | tar xzoC install-tl --strip-components 1 && \
    ./install-tl/install-tl --profile=texlive.profile --repository https://mirrors.rit.edu/CTAN/systems/texlive/tlnet
RUN ./texlive/texdir/bin/x86_64-linux/tlmgr install \
    chemfig simplekv circuitikz xstring siunitx esint mhchem chemformula \
    tikz-cd pgfplots cancel doublestroke units physics rsfs cjhebrew \
    standalone esint-type1 pgf-blur tikzlings tikzducks collection-fontsrecommended
COPY texmf.cnf ./texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/x86_64-linux/pdflatex -ini -output-format pdf '&latex preamble.tex'

FROM debian:11.0-slim AS gs
WORKDIR /app
RUN apt-get update && \
    apt-get install -y curl && \
    curl -fsSL https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs9540/ghostscript-9.54.0-linux-x86_64.tgz | tar xzoC . --strip-components 1

FROM golang:1.17.0-buster AS run
WORKDIR /app
COPY run.go .
RUN go build -ldflags '-w -s' run.go

FROM redpwn/jail:v0.0.2
COPY --from=busybox:1.33.1-glibc / /srv
COPY --from=latex /app/texlive /srv/app/texlive
COPY --from=latex /app/preamble.fmt /srv/app
COPY --from=gs /app/gs-* /srv/app/gs
COPY --from=gs /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /lib/x86_64-linux-gnu/libgcc_s.so.1 /srv/lib/
COPY --from=run /app/run /srv/app
COPY hook.sh /jail
ENV JAIL_TIME=5 JAIL_PIDS=10 JAIL_MEM=20M JAIL_CPU=800
