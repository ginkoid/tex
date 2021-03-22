FROM debian:10.7-slim AS latex

WORKDIR /app
COPY texlive.profile .
RUN apt-get update && \
    apt-get install -y curl perl && \
    mkdir install-tl && \
    curl http://dante.ctan.org/tex-archive/systems/texlive/tlnet/install-tl-unx.tar.gz | tar --strip-components 1 -xzoC install-tl && \
    ./install-tl/install-tl --profile=texlive.profile --repository=http://dante.ctan.org/tex-archive/systems/texlive/tlnet
RUN ./texlive/texdir/bin/x86_64-linux/tlmgr install \
    chemfig simplekv circuitikz xstring siunitx esint mhchem chemformula \
    tikz-cd pgfplots cancel doublestroke units physics rsfs cjhebrew \
    standalone esint-type1 pgf-blur tikzlings
COPY --chmod=744 texmf.cnf ./texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/x86_64-linux/pdflatex -ini -output-format=pdf "&latex preamble.tex"

FROM debian:10.7-slim AS gs

WORKDIR /app
RUN apt-get update && \
    apt-get install -y curl && \
    curl -L https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs9533/ghostscript-9.53.3-linux-x86_64.tgz | tar --strip-components 1 -xzoC . && \
    mv gs-* gs

FROM golang:1.14.13-buster AS run

WORKDIR /app
COPY run.go .
RUN go build -ldflags="-w -s" run.go

FROM busybox:1.32.1-glibc AS srv

COPY --from=latex /app/texlive /app/texlive
COPY --from=latex /app/preamble.fmt /app
COPY --from=gs /app/gs /app
COPY --from=run /app/run /app

FROM redpwn/jail:sha-fb3c4aa0c06ae16713c9139d3907a7cfaaa077ac

COPY --from=srv / /srv
COPY --chmod=744 hook.sh /jail
ENV JAIL_TIME 5
ENV JAIL_PIDS 10
ENV JAIL_MEM 20971520
ENV JAIL_CPU 500
