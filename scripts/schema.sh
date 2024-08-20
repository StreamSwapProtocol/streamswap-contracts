#!/bin/bash

# Define the main directory path
MAIN_DIR=$(dirname "$(realpath "$0")")/..

CONTRACTS_DIR="$MAIN_DIR/contracts"
TS_DIR="$MAIN_DIR/ts"

# Iterate over contracts
for d in "$CONTRACTS_DIR"/*; do
  if [ -d "$d" ]; then
    echo "Processing directory: $d"
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# go main directory/ts
cd "$TS_DIR" || exit
yarn install
yarn generate-ts
cd "$MAIN_DIR" || exit

# Iterate over contracts 
for d in "$CONTRACTS_DIR"/*; do
  if [ -d "$d" ]; then
    echo "Cleaning schema files in directory: $d"
    rm -rf "$d"/schema
  fi
done
