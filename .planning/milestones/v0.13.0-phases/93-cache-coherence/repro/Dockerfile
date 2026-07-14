# git-2.54 proof environment — matches CI runner git (actions/checkout reports 2.54.0).
# Host box is git 2.25.1, below the reposix >=2.34 helper floor, so ALL helper/push/pull
# litmus MUST run here. Pattern from .planning/phases/92-*/92-T4-REPRO-NOTES.md.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        software-properties-common ca-certificates sqlite3 curl \
    && add-apt-repository -y ppa:git-core/ppa \
    && apt-get update \
    && apt-get install -y --no-install-recommends git \
    && rm -rf /var/lib/apt/lists/*

# Sanity: fail the image build if we did not land on >= 2.34.
RUN git --version && test "$(git --version | grep -oE '[0-9]+\.[0-9]+' | head -1 | cut -d. -f1)" -ge 2

WORKDIR /work
