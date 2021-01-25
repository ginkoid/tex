# syntax=docker/dockerfile:1.2.1

FROM debian:10.7-slim AS nsjail

WORKDIR /app
RUN apt-get update && \
    apt-get install -y curl autoconf bison flex gcc g++ git libprotobuf-dev libnl-route-3-dev libtool make pkg-config protobuf-compiler && \
    git clone --depth 1 --branch 3.0 https://github.com/google/nsjail . && \
    make

FROM debian:10.7-slim AS latex

COPY texlive.profile /
RUN apt-get update && \
    apt-get install -y curl perl && \
    mkdir install-tl && \
    curl http://dante.ctan.org/tex-archive/systems/texlive/tlnet/install-tl-unx.tar.gz | tar --strip-components 1 -xzoC install-tl && \
    /install-tl/install-tl --profile=texlive.profile --repository=http://dante.ctan.org/tex-archive/systems/texlive/tlnet
RUN /texlive/texdir/bin/x86_64-linux/tlmgr install chemfig simplekv circuitikz xstring siunitx esint mhchem chemformula tikz-cd pgfplots cancel doublestroke units physics rsfs cjhebrew standalone esint-type1
COPY --chmod=744 texmf.cnf /texlive/texdir
COPY preamble.tex /
RUN /texlive/texdir/bin/x86_64-linux/pdflatex -ini -output-format=pdf "&latex preamble.tex"

FROM debian:10.7-slim AS ghostscript

WORKDIR /app
RUN apt-get update && \
    apt-get install -y curl && \
    curl -L https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs9533/ghostscript-9.53.3-linux-x86_64.tgz | tar --strip-components 1 -xzoC . && \
    mv gs-* gs

FROM golang:1.14.13-buster AS job

WORKDIR /app
COPY job.go .
RUN go build -ldflags="-w -s" job.go

FROM debian:10.7-slim

RUN apt-get update && \
    apt-get install --no-install-recommends -y libprotobuf17 libnl-route-3-200 && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir /app && \
    useradd -M nsjail
COPY --from=nsjail /app/nsjail /app
COPY --from=latex /texlive /app/texlive
COPY --from=latex /preamble.fmt /app
COPY --from=ghostscript /app/gs /app
COPY --from=job /app/job /app
COPY --chmod=755 nsjail.cfg run.sh /app/

CMD ["/app/run.sh"]
