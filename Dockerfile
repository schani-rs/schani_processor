from jamesnetherton/rawtherapee:latest

USER rawtherapee

COPY target/release/webservice /usr/local/bin

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/webservice"]
