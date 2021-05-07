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
    standalone esint-type1 pgf-blur tikzlings tikzducks collection-fontsrecommended
COPY --chmod=744 texmf.cnf ./texlive/texdir
COPY preamble.tex .
RUN ./texlive/texdir/bin/x86_64-linux/pdflatex -ini -output-format=pdf "&latex preamble.tex"

FROM debian:10.7-slim AS gs

WORKDIR /app
RUN apt-get update && \
    apt-get install -y curl && \
    curl -L https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs9533/ghostscript-9.53.3-linux-x86_64.tgz | tar --strip-components 1 -xzoC . && \
    mv gs-* gs

FROM golang:1.16.3-buster AS run

WORKDIR /app
COPY run.go .
RUN go build -ldflags="-w -s" run.go

FROM redpwn/jail:sha-3799217c407ee45f5d7786fb7bfdec18c9dba695

COPY --from=busybox:1.32.1-glibc / /srv
COPY --from=latex /app/texlive /srv/app/texlive
COPY --from=latex /app/preamble.fmt /srv/app
COPY --from=gs /app/gs /srv/app
COPY --from=run /app/run /srv/app
COPY --chmod=744 hook.sh /jail
ENV JAIL_TIME=5 JAIL_PIDS=10 JAIL_MEM=20971520 JAIL_CPU=500
