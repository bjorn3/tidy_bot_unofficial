#!/bin/bash

set -e

repo=$1
commit=$2

rm -rf rust/ rust-*/

curl -L $repo/archive/$commit.tar.gz | tar xz

mv rust-$commit rust
