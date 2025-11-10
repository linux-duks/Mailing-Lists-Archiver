#!/bin/bash

TARGET_DIR="./LKML5Ws"
echo "Creating $TARGET_DIR for compression"
mkdir $TARGET_DIR

for file in *.tar.gz; do tar -zxf "$file" -C $TARGET_DIR; done
