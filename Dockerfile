from jamesnetherton/rawtherapee:latest

USER root

RUN apt-get update && \
       apt-get install -y \
       libpq5 \
       --no-install-recommends

USER rawtherapee

COPY target/release/webservice /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/webservice"]
