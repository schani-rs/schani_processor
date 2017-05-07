#!/bin/bash

set -ev

rawtherapee \
    -j90 \
    -Y \
    -c $1
