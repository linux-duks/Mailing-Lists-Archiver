FROM python:3.10-slim

WORKDIR /parser

COPY ./parser/container/parser_reqs.txt /parser/requirements.txt
COPY ./parser/src /parser/src

RUN apt-get update && \
    apt-get install -y gcc python3-dev build-essential && \
    pip install --no-cache-dir -r requirements.txt && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
